use std::{fs, path::Path};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config {path}: {source}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse YAML config {path}: {source}")]
    Parse {
        path: String,
        #[source]
        source: serde_yaml::Error,
    },
    #[error("invalid config: {0}")]
    Invalid(String),
}

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
    #[serde(default = "default_motion_scale")]
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
    #[serde(default = "default_particles_gravity")]
    pub gravity: f32,
    #[serde(default = "default_particles_drag")]
    pub drag: f32,
    #[serde(default = "default_particles_thermal_lift")]
    pub thermal_lift: f32,
    #[serde(default = "default_particles_jitter")]
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

#[derive(Debug, Clone, Default)]
pub struct Overrides {
    pub backend: Option<BackendKind>,
    pub output_dir: Option<String>,
    pub export: Option<ExportMode>,
    pub sequence_format: Option<SequenceFormat>,
    pub frames: Option<u32>,
    pub duration: Option<f32>,
    pub fps: Option<f32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub seed: Option<u64>,
    pub quality: Option<Quality>,
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

    pub fn apply_overrides(&mut self, overrides: &Overrides) {
        if let Some(backend) = overrides.backend {
            self.backend.preferred = backend;
        }
        if let Some(output_dir) = &overrides.output_dir {
            self.export.output_dir = output_dir.clone();
        }
        if let Some(export) = overrides.export {
            self.export.mode = export;
        }
        if let Some(sequence_format) = overrides.sequence_format {
            self.export.sequence_format = sequence_format;
        }
        if let Some(duration) = overrides.duration {
            self.scene.duration = duration;
        }
        if let Some(fps) = overrides.fps {
            self.scene.fps = fps;
        }
        if let Some(width) = overrides.width {
            self.render.width = width;
        }
        if let Some(height) = overrides.height {
            self.render.height = height;
        }
        if let Some(seed) = overrides.seed {
            self.scene.seed = seed;
        }
        if let Some(quality) = overrides.quality {
            self.render.quality = quality;
        }
    }

    pub fn frame_count(&self, override_frames: Option<u32>) -> u32 {
        override_frames.unwrap_or_else(|| {
            let frames = (self.scene.duration * self.scene.fps).round();
            frames.max(1.0) as u32
        })
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        require(
            !self.scene.name.trim().is_empty(),
            "scene.name must not be empty",
        )?;
        range(self.scene.duration, 0.001, 24.0, "scene.duration")?;
        range(self.scene.fps, 1.0, 240.0, "scene.fps")?;
        require(self.render.width >= 16, "render.width must be at least 16")?;
        require(
            self.render.height >= 16,
            "render.height must be at least 16",
        )?;
        require(
            self.render.width <= 8192 && self.render.height <= 8192,
            "render dimensions must be <= 8192",
        )?;
        require(
            self.simulation.field_width >= 16 && self.simulation.field_height >= 16,
            "simulation field dimensions must be at least 16",
        )?;
        require(
            self.simulation.substeps > 0,
            "simulation.substeps must be positive",
        )?;
        range(
            self.simulation.motion_scale,
            0.01,
            4.0,
            "simulation.motion_scale",
        )?;
        range(
            self.simulation.velocity_decay,
            0.0,
            1.0,
            "simulation.velocity_decay",
        )?;
        range(self.simulation.vorticity, 0.0, 5.0, "simulation.vorticity")?;
        range(self.simulation.buoyancy, 0.0, 5.0, "simulation.buoyancy")?;
        range(
            self.simulation.turbulence,
            0.0,
            5.0,
            "simulation.turbulence",
        )?;
        range(self.simulation.swirl, 0.0, 5.0, "simulation.swirl")?;
        range(
            self.simulation.advection_dissipation,
            0.0,
            1.0,
            "simulation.advection_dissipation",
        )?;
        range(
            self.plasma.emission_strength,
            0.0,
            32.0,
            "plasma.emission_strength",
        )?;
        range(
            self.plasma.filament_density,
            0.0,
            8.0,
            "plasma.filament_density",
        )?;
        range(
            self.plasma.core_temperature,
            0.0,
            4.0,
            "plasma.core_temperature",
        )?;
        range(self.plasma.edge_wrapping, 0.0, 4.0, "plasma.edge_wrapping")?;
        parse_hex_color(&self.plasma.color_core)?;
        parse_hex_color(&self.plasma.color_edge)?;
        require(
            self.particles.count <= 2_000_000,
            "particles.count is too high",
        )?;
        range(self.particles.size_min, 0.01, 64.0, "particles.size_min")?;
        range(
            self.particles.size_max,
            self.particles.size_min,
            96.0,
            "particles.size_max",
        )?;
        range(self.particles.lifetime, 0.01, 60.0, "particles.lifetime")?;
        range(
            self.particles.flow_coupling,
            0.0,
            1.0,
            "particles.flow_coupling",
        )?;
        range(
            self.particles.ember_brightness,
            0.0,
            32.0,
            "particles.ember_brightness",
        )?;
        range(self.particles.gravity, 0.0, 2.0, "particles.gravity")?;
        range(self.particles.drag, 0.0, 20.0, "particles.drag")?;
        range(
            self.particles.thermal_lift,
            0.0,
            4.0,
            "particles.thermal_lift",
        )?;
        range(self.particles.jitter, 0.0, 2.0, "particles.jitter")?;
        range(self.smoke.density, 0.0, 8.0, "smoke.density")?;
        range(self.smoke.scattering, 0.0, 8.0, "smoke.scattering")?;
        range(self.smoke.absorption, 0.0, 8.0, "smoke.absorption")?;
        parse_hex_color(&self.smoke.color_cool)?;
        parse_hex_color(&self.smoke.color_warm)?;
        range(self.bloom.threshold, 0.0, 64.0, "bloom.threshold")?;
        range(self.bloom.strength, 0.0, 8.0, "bloom.strength")?;
        range(self.bloom.radius, 0.0, 8.0, "bloom.radius")?;
        require(
            (1..=8).contains(&self.bloom.pyramid_levels),
            "bloom.pyramid_levels must be 1..=8",
        )?;
        range(self.grade.exposure, -8.0, 8.0, "grade.exposure")?;
        range(self.grade.contrast, 0.0, 4.0, "grade.contrast")?;
        range(self.grade.saturation, 0.0, 4.0, "grade.saturation")?;
        range(self.grade.black_point, 0.0, 1.0, "grade.black_point")?;
        range(self.grade.vignette, 0.0, 2.0, "grade.vignette")?;
        range(self.grade.grain, 0.0, 1.0, "grade.grain")?;
        require(
            !self.export.output_dir.trim().is_empty(),
            "export.output_dir must not be empty",
        )?;
        require(self.export.crf <= 51, "export.crf must be <= 51")?;
        if self.export.sequence_format == SequenceFormat::Exr {
            return Err(ConfigError::Invalid(
                "EXR sequence export is declared for future HDR output but is not implemented in v1".to_string(),
            ));
        }
        Ok(())
    }

    pub fn smoke_cool_rgb(&self) -> Result<Rgb, ConfigError> {
        parse_hex_color(&self.smoke.color_cool)
    }

    pub fn smoke_warm_rgb(&self) -> Result<Rgb, ConfigError> {
        parse_hex_color(&self.smoke.color_warm)
    }

    pub fn plasma_core_rgb(&self) -> Result<Rgb, ConfigError> {
        parse_hex_color(&self.plasma.color_core)
    }

    pub fn plasma_edge_rgb(&self) -> Result<Rgb, ConfigError> {
        parse_hex_color(&self.plasma.color_edge)
    }
}

pub fn parse_hex_color(value: &str) -> Result<Rgb, ConfigError> {
    let hex = value.strip_prefix('#').unwrap_or(value);
    if hex.len() != 6 {
        return Err(ConfigError::Invalid(format!(
            "expected 6-digit hex color, got {value:?}"
        )));
    }
    let r = u8::from_str_radix(&hex[0..2], 16)
        .map_err(|_| ConfigError::Invalid(format!("invalid red channel in {value:?}")))?;
    let g = u8::from_str_radix(&hex[2..4], 16)
        .map_err(|_| ConfigError::Invalid(format!("invalid green channel in {value:?}")))?;
    let b = u8::from_str_radix(&hex[4..6], 16)
        .map_err(|_| ConfigError::Invalid(format!("invalid blue channel in {value:?}")))?;
    Ok(Rgb {
        r: f32::from(r) / 255.0,
        g: f32::from(g) / 255.0,
        b: f32::from(b) / 255.0,
    })
}

fn range(value: f32, min: f32, max: f32, name: &str) -> Result<(), ConfigError> {
    require(
        value.is_finite() && value >= min && value <= max,
        &format!("{name} must be in range {min}..={max}"),
    )
}

fn require(condition: bool, message: &str) -> Result<(), ConfigError> {
    if condition {
        Ok(())
    } else {
        Err(ConfigError::Invalid(message.to_string()))
    }
}

fn default_motion_scale() -> f32 {
    1.0
}

fn default_particles_gravity() -> f32 {
    0.12
}

fn default_particles_drag() -> f32 {
    3.5
}

fn default_particles_thermal_lift() -> f32 {
    0.8
}

fn default_particles_jitter() -> f32 {
    0.15
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_valid_config() {
        let config: Config =
            serde_yaml::from_str(include_str!("../../../configs/inferno.production.yaml"))
                .expect("sample config parses");
        config.validate().expect("sample config validates");
    }

    #[test]
    fn rejects_missing_groups() {
        let err = serde_yaml::from_str::<Config>("scene:\n  name: nope\n")
            .expect_err("missing groups should fail");
        assert!(err.to_string().contains("missing field"));
    }

    #[test]
    fn parses_hex_colors() {
        assert_eq!(
            parse_hex_color("#102040").unwrap(),
            Rgb {
                r: 16.0 / 255.0,
                g: 32.0 / 255.0,
                b: 64.0 / 255.0
            }
        );
    }
}
