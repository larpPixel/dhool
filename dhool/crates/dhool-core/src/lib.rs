use std::{
    fs,
    path::Path,
    process::Command,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub backend: String,
    pub available: bool,
    pub name: String,
    pub driver_version: String,
    pub runtime_version: String,
    pub global_memory_bytes: u64,
    pub multiprocessor_count: i32,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FrameTiming {
    pub frame_index: u32,
    pub gpu_ms: f32,
    pub readback_ms: f32,
    pub total_ms: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingSummary {
    pub frames: u32,
    pub total_wall_ms: f32,
    pub avg_gpu_ms: f32,
    pub avg_readback_ms: f32,
    pub avg_total_ms: f32,
    pub max_total_ms: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitState {
    pub commit: Option<String>,
    pub dirty: bool,
    pub status_porcelain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderMetadata {
    pub renderer: String,
    pub command: String,
    pub created_at: DateTime<Utc>,
    pub scene_name: String,
    pub seed: u64,
    pub backend: String,
    pub width: u32,
    pub height: u32,
    pub fps: f32,
    pub frames: u32,
    pub device: DeviceInfo,
    pub git: GitState,
    pub timings: TimingSummary,
    pub outputs: Vec<String>,
    pub known_limitations: Vec<String>,
}

#[derive(Debug)]
pub struct Stopwatch {
    started: Instant,
}

impl Stopwatch {
    pub fn start() -> Self {
        Self {
            started: Instant::now(),
        }
    }

    pub fn elapsed_ms(&self) -> f32 {
        duration_ms(self.started.elapsed())
    }
}

pub fn summarize_timings(frames: &[FrameTiming], wall_ms: f32) -> TimingSummary {
    if frames.is_empty() {
        return TimingSummary {
            frames: 0,
            total_wall_ms: wall_ms,
            avg_gpu_ms: 0.0,
            avg_readback_ms: 0.0,
            avg_total_ms: 0.0,
            max_total_ms: 0.0,
        };
    }
    let len = frames.len() as f32;
    let gpu = frames.iter().map(|t| t.gpu_ms).sum::<f32>() / len;
    let readback = frames.iter().map(|t| t.readback_ms).sum::<f32>() / len;
    let total = frames.iter().map(|t| t.total_ms).sum::<f32>() / len;
    let max_total = frames
        .iter()
        .map(|t| t.total_ms)
        .fold(0.0_f32, |acc, value| acc.max(value));
    TimingSummary {
        frames: frames.len() as u32,
        total_wall_ms: wall_ms,
        avg_gpu_ms: gpu,
        avg_readback_ms: readback,
        avg_total_ms: total,
        max_total_ms: max_total,
    }
}

pub fn capture_git_state(repo: impl AsRef<Path>) -> GitState {
    let repo = repo.as_ref();
    let commit = run_git(repo, &["rev-parse", "HEAD"]).ok();
    let status = run_git(repo, &["status", "--porcelain"]).unwrap_or_default();
    GitState {
        commit,
        dirty: !status.trim().is_empty(),
        status_porcelain: status,
    }
}

pub fn write_json(path: impl AsRef<Path>, value: &impl Serialize) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let bytes = serde_json::to_vec_pretty(value).context("failed to serialize JSON")?;
    fs::write(path, bytes).with_context(|| format!("failed to write {}", path.display()))
}

pub fn copy_text(path: impl AsRef<Path>, contents: &str) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(path, contents).with_context(|| format!("failed to write {}", path.display()))
}

pub fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

pub fn seeded_unit_f32(seed: u64, index: u64) -> f32 {
    let bits = splitmix64(seed ^ index);
    ((bits >> 40) as f32) / ((1_u64 << 24) as f32)
}

fn duration_ms(duration: Duration) -> f32 {
    duration.as_secs_f32() * 1000.0
}

fn run_git(repo: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .with_context(|| format!("failed to run git {}", args.join(" ")))?;
    if !output.status.success() {
        anyhow::bail!("git {} failed", args.join(" "));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_rng_is_stable() {
        assert_eq!(seeded_unit_f32(781278, 42), seeded_unit_f32(781278, 42));
        assert_ne!(seeded_unit_f32(781278, 42), seeded_unit_f32(781278, 43));
    }

    #[test]
    fn summarizes_timings() {
        let summary = summarize_timings(
            &[
                FrameTiming {
                    frame_index: 0,
                    gpu_ms: 2.0,
                    readback_ms: 3.0,
                    total_ms: 8.0,
                },
                FrameTiming {
                    frame_index: 1,
                    gpu_ms: 4.0,
                    readback_ms: 5.0,
                    total_ms: 12.0,
                },
            ],
            21.0,
        );
        assert_eq!(summary.frames, 2);
        assert_eq!(summary.avg_gpu_ms, 3.0);
        assert_eq!(summary.max_total_ms, 12.0);
    }
}
