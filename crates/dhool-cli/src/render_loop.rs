use std::{
    collections::BTreeMap,
    env,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU32, Ordering},
        mpsc,
    },
    thread,
};

use anyhow::{Context, Result};
use dhool_config::Config;
use dhool_core::{
    FrameTiming, automatic_cuda_worker_count, cap_workers_to_frame_count, manual_cuda_worker_count,
    resolution_limited_workers,
};
use dhool_cuda::{RenderedFrame, device_info, render_frame};

enum RenderWorkerResult {
    Frame(RenderedFrame),
    Error { frame_index: u32, message: String },
}

pub(crate) fn render_frames_ordered(
    command: &str,
    config: &Config,
    frame_count: u32,
    mut on_frame: impl FnMut(RenderedFrame) -> Result<()>,
) -> Result<Vec<FrameTiming>> {
    if frame_count == 0 {
        return Ok(Vec::new());
    }

    let worker_count = cli_render_worker_count(config, frame_count);
    println!("{command}: rendering {frame_count} frame(s) with {worker_count} CUDA worker(s)");

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
                        Ok(frame) => RenderWorkerResult::Frame(frame),
                        Err(err) => {
                            stop.store(true, Ordering::Relaxed);
                            RenderWorkerResult::Error {
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
                RenderWorkerResult::Frame(frame) => {
                    pending.insert(frame.timing.frame_index, frame);
                }
                RenderWorkerResult::Error {
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

fn cli_render_worker_count(config: &Config, frame_count: u32) -> usize {
    let frame_count = frame_count.max(1) as usize;
    if let Ok(value) = env::var("DHOOL_CLI_GPU_WORKERS") {
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
