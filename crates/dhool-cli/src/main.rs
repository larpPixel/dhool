use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU32, Ordering},
        mpsc,
    },
    thread,
};

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use dhool_config::{BackendKind, Config, ExportMode, Overrides, Quality, SequenceFormat};
use dhool_core::{
    FrameTiming, RenderMetadata, Stopwatch, capture_git_state, copy_text, summarize_timings,
    write_json,
};
use dhool_cuda::{RenderedFrame, device_info, known_limitations, render_frame};
use dhool_export::{FfmpegPipe, plan_outputs, write_png, write_sequence_frame};

const MAX_AUTO_CLI_WORKERS: usize = 6;
const MAX_MANUAL_CLI_WORKERS: usize = 16;

#[derive(Debug, Parser)]
#[command(name = "dhool", about = "Rust/CUDA cinematic background renderer")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    DeviceInfo,
    Validate(RenderArgs),
    Still(RenderArgs),
    Preview(RenderArgs),
    Final(RenderArgs),
    Benchmark(RenderArgs),
}

#[derive(Debug, Clone, Args)]
struct RenderArgs {
    config: PathBuf,
    #[arg(long, value_enum)]
    backend: Option<BackendKind>,
    #[arg(long)]
    output_dir: Option<String>,
    #[arg(long, value_enum)]
    export: Option<ExportMode>,
    #[arg(long, value_enum)]
    sequence_format: Option<SequenceFormat>,
    #[arg(long)]
    frames: Option<u32>,
    #[arg(long)]
    duration: Option<f32>,
    #[arg(long)]
    fps: Option<f32>,
    #[arg(long)]
    width: Option<u32>,
    #[arg(long)]
    height: Option<u32>,
    #[arg(long)]
    seed: Option<u64>,
    #[arg(long, value_enum)]
    quality: Option<Quality>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::DeviceInfo => print_device_info(),
        Command::Validate(args) => validate(args),
        Command::Still(args) => still(args),
        Command::Preview(args) => movie("preview", args, true),
        Command::Final(args) => movie("final", args, true),
        Command::Benchmark(args) => benchmark(args),
    }
}

fn print_device_info() -> Result<()> {
    let info = device_info();
    println!("{}", serde_json::to_string_pretty(&info)?);
    anyhow::ensure!(info.available, "CUDA device is not available");
    Ok(())
}

fn validate(args: RenderArgs) -> Result<()> {
    let config = load_effective_config(&args)?;
    println!(
        "valid config: scene={} backend={:?} size={}x{} duration={:.3}s fps={:.3}",
        config.scene.name,
        config.backend.preferred,
        config.render.width,
        config.render.height,
        config.scene.duration,
        config.scene.fps
    );
    Ok(())
}

fn still(args: RenderArgs) -> Result<()> {
    let config = load_effective_config(&args)?;
    ensure_cuda_backend(&config, "still")?;
    let output_dir = PathBuf::from(&config.export.output_dir);
    fs::create_dir_all(&output_dir)
        .with_context(|| format!("failed to create {}", output_dir.display()))?;

    let wall = Stopwatch::start();
    let frame = render_frame(&config, 0, 1)?;
    let still_path = output_dir.join(format!("{}_still.png", sanitize_stem(&config.scene.name)));
    write_png(&still_path, frame.width, frame.height, &frame.rgba)?;
    let outputs = vec![still_path.display().to_string()];
    write_run_sidecars(
        "still",
        &config,
        1,
        &[frame.timing],
        wall.elapsed_ms(),
        &outputs,
    )?;
    println!(
        "wrote {} (gpu {:.3} ms, readback {:.3} ms)",
        still_path.display(),
        frame.timing.gpu_ms,
        frame.timing.readback_ms
    );
    Ok(())
}

fn movie(command: &str, args: RenderArgs, include_sequence: bool) -> Result<()> {
    let config = load_effective_config(&args)?;
    ensure_cuda_backend(&config, command)?;
    let frame_count = config.frame_count(args.frames);
    let planned = plan_outputs(&config.export, command, include_sequence);
    fs::create_dir_all(&planned.output_dir)
        .with_context(|| format!("failed to create {}", planned.output_dir.display()))?;
    if let Some(sequence_dir) = &planned.sequence_dir {
        fs::create_dir_all(sequence_dir)
            .with_context(|| format!("failed to create {}", sequence_dir.display()))?;
    }

    let mut pipe = match &planned.video_path {
        Some(path) => Some(FfmpegPipe::start(
            path,
            config.render.width,
            config.render.height,
            config.scene.fps,
            &config.export,
        )?),
        None => None,
    };

    let wall = Stopwatch::start();
    let mut outputs = Vec::new();
    let timings = render_frames_ordered(command, &config, frame_count, |frame| {
        let frame_index = frame.timing.frame_index;
        if let Some(sequence_dir) = &planned.sequence_dir {
            let path = write_sequence_frame(
                sequence_dir,
                frame_index,
                config.export.sequence_format,
                frame.width,
                frame.height,
                &frame.rgba,
            )?;
            if frame_index == 0 {
                outputs.push(path.display().to_string());
            }
        }
        if let Some(pipe) = pipe.as_mut() {
            pipe.write_frame(&frame.rgba)?;
        }
        if frame_count <= 5
            || frame_index == 0
            || frame_index + 1 == frame_count
            || (frame_index + 1) % 10 == 0
        {
            println!(
                "{command}: frame {}/{} gpu {:.3} ms readback {:.3} ms",
                frame_index + 1,
                frame_count,
                frame.timing.gpu_ms,
                frame.timing.readback_ms
            );
        }
        Ok(())
    })?;
    if let Some(pipe) = pipe {
        outputs.push(pipe.finish()?.display().to_string());
    }
    if let Some(sequence_dir) = &planned.sequence_dir {
        outputs.push(sequence_dir.display().to_string());
    }
    write_run_sidecars(
        command,
        &config,
        frame_count,
        &timings,
        wall.elapsed_ms(),
        &outputs,
    )?;
    println!(
        "{command}: wrote {} frame(s) to {}",
        frame_count,
        planned.output_dir.display()
    );
    Ok(())
}

fn benchmark(args: RenderArgs) -> Result<()> {
    let mut args = args;
    if args.output_dir.is_none() {
        args.output_dir = Some("outputs/cuda_probe".to_string());
    }
    let config = load_effective_config(&args)?;
    ensure_cuda_backend(&config, "benchmark")?;
    let frame_count = config.frame_count(args.frames.or(Some(1)));
    let output_dir = PathBuf::from(&config.export.output_dir);
    fs::create_dir_all(&output_dir)
        .with_context(|| format!("failed to create {}", output_dir.display()))?;

    let wall = Stopwatch::start();
    let mut outputs = Vec::new();
    let timings = render_frames_ordered("benchmark", &config, frame_count, |frame| {
        let frame_index = frame.timing.frame_index;
        if frame_index == 0 {
            let path = output_dir.join("cuda_probe.png");
            write_png(&path, frame.width, frame.height, &frame.rgba)?;
            outputs.push(path.display().to_string());
        }
        Ok(())
    })?;
    let wall_ms = wall.elapsed_ms();
    let metadata = build_metadata(
        "benchmark",
        &config,
        frame_count,
        &timings,
        wall_ms,
        &outputs,
    )?;
    let report_path = output_dir.join("cuda_feasibility_report.json");
    write_json(&report_path, &metadata)?;
    write_effective_config(&output_dir, &config)?;
    println!(
        "benchmark: {} frame(s), avg gpu {:.3} ms, avg readback {:.3} ms, report {}",
        frame_count,
        metadata.timings.avg_gpu_ms,
        metadata.timings.avg_readback_ms,
        report_path.display()
    );
    Ok(())
}

enum RenderWorkerResult {
    Frame(RenderedFrame),
    Error { frame_index: u32, message: String },
}

fn render_frames_ordered(
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

    let render_result = thread::scope(|scope| -> Result<Vec<FrameTiming>> {
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
    });

    render_result
}

fn cli_render_worker_count(config: &Config, frame_count: u32) -> usize {
    let frame_count = frame_count.max(1) as usize;
    if let Ok(value) = env::var("DHOOL_CLI_GPU_WORKERS") {
        if let Ok(parsed) = value.parse::<usize>() {
            return parsed.clamp(1, MAX_MANUAL_CLI_WORKERS).min(frame_count);
        }
    }

    let info = device_info();
    let multiprocessors = info.multiprocessor_count.max(1) as usize;
    let auto_workers = ((multiprocessors + 23) / 24).clamp(2, MAX_AUTO_CLI_WORKERS);
    resolution_limited_workers(config, auto_workers).min(frame_count)
}

fn resolution_limited_workers(config: &Config, worker_count: usize) -> usize {
    let pixels = u64::from(config.render.width) * u64::from(config.render.height);
    let resolution_cap = if pixels <= 1920 * 1080 {
        worker_count
    } else if pixels <= 2560 * 1440 {
        worker_count.min(4)
    } else if pixels <= 3840 * 2160 {
        worker_count.min(3)
    } else {
        worker_count.min(2)
    };
    resolution_cap.max(1)
}

fn load_effective_config(args: &RenderArgs) -> Result<Config> {
    let mut config = Config::load(&args.config)?;
    config.apply_overrides(&Overrides {
        backend: args.backend,
        output_dir: args.output_dir.clone(),
        export: args.export,
        sequence_format: args.sequence_format,
        frames: args.frames,
        duration: args.duration,
        fps: args.fps,
        width: args.width,
        height: args.height,
        seed: args.seed,
        quality: args.quality,
    });
    config.validate()?;
    Ok(config)
}

fn ensure_cuda_backend(config: &Config, command: &str) -> Result<()> {
    anyhow::ensure!(
        config.backend.preferred == BackendKind::Cuda,
        "{command} requires --backend cuda; cpu-reference is only for setup validation tests"
    );
    let info = device_info();
    anyhow::ensure!(
        info.available,
        "CUDA is not available: {}",
        info.error
            .unwrap_or_else(|| "unknown CUDA initialization failure".to_string())
    );
    Ok(())
}

fn write_run_sidecars(
    command: &str,
    config: &Config,
    frames: u32,
    timings: &[dhool_core::FrameTiming],
    wall_ms: f32,
    outputs: &[String],
) -> Result<()> {
    let output_dir = PathBuf::from(&config.export.output_dir);
    let metadata = build_metadata(command, config, frames, timings, wall_ms, outputs)?;
    write_json(output_dir.join("metadata.json"), &metadata)?;
    write_effective_config(&output_dir, config)
}

fn write_effective_config(output_dir: &Path, config: &Config) -> Result<()> {
    let yaml = serde_yaml::to_string(config).context("failed to serialize effective config")?;
    copy_text(output_dir.join("config.effective.yaml"), &yaml)
}

fn build_metadata(
    command: &str,
    config: &Config,
    frames: u32,
    timings: &[dhool_core::FrameTiming],
    wall_ms: f32,
    outputs: &[String],
) -> Result<RenderMetadata> {
    let cwd = env::current_dir().context("failed to read current directory")?;
    Ok(RenderMetadata {
        renderer: "dhool 0.1.0".to_string(),
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

fn sanitize_stem(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}
