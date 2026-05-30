use std::{fs, path::Path};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::ConfigError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub scene: SceneConfig,
    pub render: RenderConfig,
    pub backend: BackendConfig,
    pub simulation: SimulationConfig,
    pub plasma: PlasmaConfig,
    pub particles: ParticlesConfig,
    pub smoke: SmokeConfig,
    pub bloom: BloomConfig,
    pub grade: GradeConfig,
    pub export: ExportConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneConfig {
    pub name: String,
    pub seed: u64,
    pub duration: f32,
    pub fps: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderConfig {
    pub width: u32,
    pub height: u32,
    pub quality: Quality,
    pub internal_format: InternalFormat,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Quality {
    Preview,
    Final,
    Ultra,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum InternalFormat {
    Rgba16f,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    pub preferred: BackendKind,
    pub require_gpu: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum BackendKind {
    Cuda,
    CpuReference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub field_width: u32,
    pub field_height: u32,
    pub substeps: u32,
    #[serde(default = "crate::defaults::motion_scale")]
    pub motion_scale: f32,
    pub velocity_decay: f32,
    pub vorticity: f32,
    pub buoyancy: f32,
    pub turbulence: f32,
    pub swirl: f32,
    pub advection_dissipation: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlasmaConfig {
    pub emission_strength: f32,
    pub filament_density: f32,
    pub core_temperature: f32,
    pub edge_wrapping: f32,
    pub color_core: String,
    pub color_edge: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticlesConfig {
    pub count: u32,
    pub size_min: f32,
    pub size_max: f32,
    pub lifetime: f32,
    pub flow_coupling: f32,
    pub ember_brightness: f32,
    #[serde(default = "crate::defaults::particles_gravity")]
    pub gravity: f32,
    #[serde(default = "crate::defaults::particles_drag")]
    pub drag: f32,
    #[serde(default = "crate::defaults::particles_thermal_lift")]
    pub thermal_lift: f32,
    #[serde(default = "crate::defaults::particles_jitter")]
    pub jitter: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmokeConfig {
    pub density: f32,
    pub scattering: f32,
    pub absorption: f32,
    pub color_cool: String,
    pub color_warm: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Rgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BloomConfig {
    pub threshold: f32,
    pub strength: f32,
    pub radius: f32,
    pub pyramid_levels: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradeConfig {
    pub exposure: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub black_point: f32,
    pub vignette: f32,
    pub grain: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    pub mode: ExportMode,
    pub sequence_format: SequenceFormat,
    pub video_codec: String,
    pub crf: u8,
    pub preset: String,
    pub output_dir: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ExportMode {
    Pipe,
    Sequence,
    Both,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SequenceFormat {
    Png,
    Exr,
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path_ref = path.as_ref();
        let path_display = path_ref.display().to_string();
        let bytes = fs::read(path_ref).map_err(|source| ConfigError::Read {
            path: path_display.clone(),
            source,
        })?;
        let config = serde_yaml::from_slice(&bytes).map_err(|source| ConfigError::Parse {
            path: path_display,
            source,
        })?;
        Ok(config)
    }

    pub fn frame_count(&self, override_frames: Option<u32>) -> u32 {
        override_frames.unwrap_or_else(|| {
            let frames = (self.scene.duration * self.scene.fps).round();
            frames.max(1.0) as u32
        })
    }
}
