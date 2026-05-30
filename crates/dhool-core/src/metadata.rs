use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{DeviceInfo, GitState, TimingSummary};

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
