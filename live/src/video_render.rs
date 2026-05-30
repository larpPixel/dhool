use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU32, Ordering},
        mpsc::{self, Receiver, Sender},
    },
    thread,
};

use anyhow::{Context, Result};
use dhool_config::{BackendKind, Config};
use dhool_core::{
    FrameTiming, RenderMetadata, Stopwatch, automatic_cuda_worker_count,
    cap_workers_to_frame_count, capture_git_state, copy_text, manual_cuda_worker_count,
    resolution_limited_workers, summarize_timings, write_json,
};
use dhool_cuda::{RenderedFrame, device_info, known_limitations, render_frame};
use dhool_export::FfmpegPipe;

#[derive(Debug)]
pub enum VideoRenderProgress {
    Started {
        frame_count: u32,
        output_path: PathBuf,
    },
    Frame {
        frame_index: u32,
        frame_count: u32,
        gpu_ms: f32,
        readback_ms: f32,
    },
    Finished {
        frame_count: u32,
        wall_ms: f32,
        outputs: Vec<String>,
    },
    Error(String),
}

enum OrderedRenderResult {
    Frame(RenderedFrame),
    Error { frame_index: u32, message: String },
}

pub fn spawn_video_render(
    config: Config,
    output_path: PathBuf,
    repaint: egui::Context,
) -> Receiver<VideoRenderProgress> {
    let (tx, rx) = mpsc::channel();
    thread::Builder::new()
        .name("dhool-live-video-render".to_string())
        .spawn(move || {
            if let Err(err) = render_video(config, output_path, &tx, &repaint) {
                send_progress(&tx, &repaint, VideoRenderProgress::Error(err.to_string()));
            }
        })
        .expect("failed to spawn live video render thread");
    rx
}

fn render_video(
    config: Config,
    output_path: PathBuf,
    progress_tx: &Sender<VideoRenderProgress>,
    repaint: &egui::Context,
) -> Result<()> {
    config.validate()?;
    anyhow::ensure!(
        config.backend.preferred == BackendKind::Cuda,
        "video rendering requires the CUDA backend"
    );
    let info = device_info();
    anyhow::ensure!(
        info.available,
        "CUDA is not available: {}",
        info.error
            .unwrap_or_else(|| "unknown CUDA initialization failure".to_string())
    );

    let frame_count = config.frame_count(None).max(1);
    let output_dir = output_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    fs::create_dir_all(&output_dir)
        .with_context(|| format!("failed to create {}", output_dir.display()))?;

    send_progress(
        progress_tx,
        repaint,
        VideoRenderProgress::Started {
            frame_count,
            output_path: output_path.clone(),
        },
    );

    let mut pipe = FfmpegPipe::start(
        &output_path,
        config.render.width,
        config.render.height,
        config.scene.fps,
        &config.export,
    )?;
    let wall = Stopwatch::start();
    let timings = render_frames_ordered(&config, frame_count, |frame| {
        let timing = frame.timing;
        pipe.write_frame(&frame.rgba)?;
        send_progress(
            progress_tx,
            repaint,
            VideoRenderProgress::Frame {
                frame_index: timing.frame_index,
                frame_count,
                gpu_ms: timing.gpu_ms,
                readback_ms: timing.readback_ms,
            },
        );
        Ok(())
    })?;
    let video_path = pipe.finish()?;
    let wall_ms = wall.elapsed_ms();
    let outputs = vec![video_path.display().to_string()];
    write_run_sidecars(
        &output_dir,
        "live-render",
        &config,
        frame_count,
        &timings,
        wall_ms,
        &outputs,
    )?;
    send_progress(
        progress_tx,
        repaint,
        VideoRenderProgress::Finished {
            frame_count,
            wall_ms,
            outputs,
        },
    );
    Ok(())
}

fn render_frames_ordered(
    config: &Config,
    frame_count: u32,
    mut on_frame: impl FnMut(RenderedFrame) -> Result<()>,
) -> Result<Vec<FrameTiming>> {
    if frame_count == 0 {
        return Ok(Vec::new());
    }

    let worker_count = live_video_worker_count(config, frame_count);
    let next_frame = Arc::new(AtomicU32::new(0));
    let stop = Arc::new(AtomicBool::new(false));
    let (result_tx, result_rx) = mpsc::channel();

    thread::scope(|scope| -> Result<Vec<FrameTiming>> {
        for _ in 0..worker_count {
            let config = config.clone();
            let result_tx = result_tx.clone();
            let next_frame = Arc::clone(&next_frame);
            let stop = Arc::clone(&stop);
            scope.spawn(move || {
                while !stop.load(Ordering::Relaxed) {
                    let frame_index = next_frame.fetch_add(1, Ordering::Relaxed);
                    if frame_index >= frame_count {
                        break;
                    }

                    let result = match render_frame(&config, frame_index, frame_count) {
                        Ok(frame) => OrderedRenderResult::Frame(frame),
                        Err(err) => {
                            stop.store(true, Ordering::Relaxed);
                            OrderedRenderResult::Error {
                                frame_index,
                                message: err.to_string(),
                            }
                        }
                    };

                    if result_tx.send(result).is_err() {
                        break;
                    }
                }
            });
        }
        drop(result_tx);

        let mut pending = BTreeMap::new();
        let mut timings = Vec::with_capacity(frame_count as usize);
        let mut next_output_frame = 0_u32;

        while next_output_frame < frame_count {
            match result_rx
                .recv()
                .context("render workers stopped before completing frames")?
            {
                OrderedRenderResult::Frame(frame) => {
                    pending.insert(frame.timing.frame_index, frame);
                }
                OrderedRenderResult::Error {
                    frame_index,
                    message,
                } => {
                    stop.store(true, Ordering::Relaxed);
                    anyhow::bail!(
                        "CUDA render failed for frame {}: {message}",
                        frame_index + 1
                    );
                }
            }

            while let Some(frame) = pending.remove(&next_output_frame) {
                timings.push(frame.timing);
                if let Err(err) = on_frame(frame) {
                    stop.store(true, Ordering::Relaxed);
                    return Err(err);
                }
                next_output_frame += 1;
            }
        }

        Ok(timings)
    })
}

fn live_video_worker_count(config: &Config, frame_count: u32) -> usize {
    let frame_count = frame_count.max(1) as usize;
    if let Ok(value) = env::var("DHOOL_LIVE_VIDEO_GPU_WORKERS") {
        if let Some(parsed) = manual_cuda_worker_count(&value, Some(frame_count)) {
            return parsed;
        }
    }

    let info = device_info();
    let auto_workers = automatic_cuda_worker_count(info.multiprocessor_count);
    cap_workers_to_frame_count(
        resolution_limited_workers(config.render.width, config.render.height, auto_workers),
        Some(frame_count),
    )
}

fn write_run_sidecars(
    output_dir: &Path,
    command: &str,
    config: &Config,
    frames: u32,
    timings: &[FrameTiming],
    wall_ms: f32,
    outputs: &[String],
) -> Result<()> {
    let metadata = build_metadata(command, config, frames, timings, wall_ms, outputs)?;
    write_json(output_dir.join("metadata.json"), &metadata)?;
    let yaml = serde_yaml::to_string(config).context("failed to serialize effective config")?;
    copy_text(output_dir.join("config.effective.yaml"), &yaml)
}

fn build_metadata(
    command: &str,
    config: &Config,
    frames: u32,
    timings: &[FrameTiming],
    wall_ms: f32,
    outputs: &[String],
) -> Result<RenderMetadata> {
    let cwd = env::current_dir().context("failed to read current directory")?;
    Ok(RenderMetadata {
        renderer: "dhool-live 0.1.0".to_string(),
        command: command.to_string(),
        created_at: chrono::Utc::now(),
        scene_name: config.scene.name.clone(),
        seed: config.scene.seed,
        backend: "cuda".to_string(),
        width: config.render.width,
        height: config.render.height,
        fps: config.scene.fps,
        frames,
        device: device_info(),
        git: capture_git_state(cwd),
        timings: summarize_timings(timings, wall_ms),
        outputs: outputs.to_vec(),
        known_limitations: known_limitations(),
    })
}

fn send_progress(
    tx: &Sender<VideoRenderProgress>,
    repaint: &egui::Context,
    progress: VideoRenderProgress,
) {
    let _ = tx.send(progress);
    repaint.request_repaint();
}
