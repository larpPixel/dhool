use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use dhool_core::{Stopwatch, sanitize_stem, write_json};
use dhool_cuda::device_info;
use dhool_export::{FfmpegPipe, plan_outputs, write_png, write_sequence_frame};

use crate::{
    args::RenderArgs,
    config::{ensure_cuda_backend, load_effective_config},
    render_loop::render_frames_ordered,
    sidecars::{build_metadata, write_effective_config, write_run_sidecars},
};

pub(crate) fn print_device_info() -> Result<()> {
    let info = device_info();
    println!("{}", serde_json::to_string_pretty(&info)?);
    anyhow::ensure!(info.available, "CUDA device is not available");
    Ok(())
}

pub(crate) fn validate(args: RenderArgs) -> Result<()> {
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

pub(crate) fn still(args: RenderArgs) -> Result<()> {
    let config = load_effective_config(&args)?;
    ensure_cuda_backend(&config, "still")?;
    let output_dir = PathBuf::from(&config.export.output_dir);
    fs::create_dir_all(&output_dir)
        .with_context(|| format!("failed to create {}", output_dir.display()))?;

    let wall = Stopwatch::start();
    let frame = dhool_cuda::render_frame(&config, 0, 1)?;
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

pub(crate) fn movie(command: &str, args: RenderArgs, include_sequence: bool) -> Result<()> {
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

pub(crate) fn benchmark(args: RenderArgs) -> Result<()> {
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
