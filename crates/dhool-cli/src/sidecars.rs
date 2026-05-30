use std::{
    env,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use dhool_config::Config;
use dhool_core::{
    FrameTiming, RenderMetadata, capture_git_state, copy_text, summarize_timings, write_json,
};
use dhool_cuda::{device_info, known_limitations};

pub(crate) fn write_run_sidecars(
    command: &str,
    config: &Config,
    frames: u32,
    timings: &[FrameTiming],
    wall_ms: f32,
    outputs: &[String],
) -> Result<()> {
    let output_dir = PathBuf::from(&config.export.output_dir);
    let metadata = build_metadata(command, config, frames, timings, wall_ms, outputs)?;
    write_json(output_dir.join("metadata.json"), &metadata)?;
    write_effective_config(&output_dir, config)
}

pub(crate) fn write_effective_config(output_dir: &Path, config: &Config) -> Result<()> {
    let yaml = serde_yaml::to_string(config).context("failed to serialize effective config")?;
    copy_text(output_dir.join("config.effective.yaml"), &yaml)
}

pub(crate) fn build_metadata(
    command: &str,
    config: &Config,
    frames: u32,
    timings: &[FrameTiming],
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
