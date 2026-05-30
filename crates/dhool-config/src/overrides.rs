use crate::{BackendKind, Config, ExportMode, Quality, SequenceFormat};

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applies_render_overrides() {
        let mut config: Config =
            serde_yaml::from_str(include_str!("../../../configs/inferno.production.yaml"))
                .expect("sample config parses");
        config.apply_overrides(&Overrides {
            width: Some(128),
            height: Some(72),
            seed: Some(7),
            quality: Some(Quality::Preview),
            ..Overrides::default()
        });
        assert_eq!(config.render.width, 128);
        assert_eq!(config.render.height, 72);
        assert_eq!(config.scene.seed, 7);
        assert_eq!(config.render.quality, Quality::Preview);
    }
}
