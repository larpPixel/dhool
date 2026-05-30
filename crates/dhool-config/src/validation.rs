use crate::{
    BLOOM_PYRAMID_LEVELS_MAX, BLOOM_PYRAMID_LEVELS_MIN, BLOOM_RADIUS_MAX, BLOOM_RADIUS_MIN,
    BLOOM_STRENGTH_MAX, BLOOM_STRENGTH_MIN, BLOOM_THRESHOLD_MAX, BLOOM_THRESHOLD_MIN, Config,
    ConfigError, EXPORT_CRF_MAX, GRADE_BLACK_POINT_MAX, GRADE_BLACK_POINT_MIN, GRADE_CONTRAST_MAX,
    GRADE_CONTRAST_MIN, GRADE_EXPOSURE_MAX, GRADE_EXPOSURE_MIN, GRADE_GRAIN_MAX, GRADE_GRAIN_MIN,
    GRADE_SATURATION_MAX, GRADE_SATURATION_MIN, GRADE_VIGNETTE_MAX, GRADE_VIGNETTE_MIN,
    MOTION_SCALE_MAX, MOTION_SCALE_MIN, PARTICLES_COUNT_MAX, PARTICLES_DRAG_MAX,
    PARTICLES_DRAG_MIN, PARTICLES_EMBER_BRIGHTNESS_MAX, PARTICLES_EMBER_BRIGHTNESS_MIN,
    PARTICLES_GRAVITY_MAX, PARTICLES_GRAVITY_MIN, PARTICLES_JITTER_MAX, PARTICLES_JITTER_MIN,
    PARTICLES_LIFETIME_MAX, PARTICLES_LIFETIME_MIN, PARTICLES_SIZE_MAX_MAX, PARTICLES_SIZE_MIN_MAX,
    PARTICLES_SIZE_MIN_MIN, PARTICLES_THERMAL_LIFT_MAX, PARTICLES_THERMAL_LIFT_MIN,
    PLASMA_CORE_TEMPERATURE_MAX, PLASMA_CORE_TEMPERATURE_MIN, PLASMA_EDGE_WRAPPING_MAX,
    PLASMA_EDGE_WRAPPING_MIN, PLASMA_EMISSION_MAX, PLASMA_EMISSION_MIN,
    PLASMA_FILAMENT_DENSITY_MAX, PLASMA_FILAMENT_DENSITY_MIN, RENDER_DIMENSION_MAX,
    RENDER_DIMENSION_MIN, Rgb, SCENE_DURATION_MAX, SCENE_DURATION_MIN, SCENE_FPS_MAX,
    SCENE_FPS_MIN, SIMULATION_FIELD_DIMENSION_MIN, SIMULATION_FORCE_MAX, SIMULATION_FORCE_MIN,
    SIMULATION_SUBSTEPS_MIN, SMOKE_MAX, SMOKE_MIN, SequenceFormat, UNIT_INTERVAL_MAX,
    UNIT_INTERVAL_MIN,
};

impl Config {
    pub fn validate(&self) -> Result<(), ConfigError> {
        require(
            !self.scene.name.trim().is_empty(),
            "scene.name must not be empty",
        )?;
        range(
            self.scene.duration,
            SCENE_DURATION_MIN,
            SCENE_DURATION_MAX,
            "scene.duration",
        )?;
        range(self.scene.fps, SCENE_FPS_MIN, SCENE_FPS_MAX, "scene.fps")?;
        require(
            self.render.width >= RENDER_DIMENSION_MIN,
            "render.width must be at least 16",
        )?;
        require(
            self.render.height >= RENDER_DIMENSION_MIN,
            "render.height must be at least 16",
        )?;
        require(
            self.render.width <= RENDER_DIMENSION_MAX && self.render.height <= RENDER_DIMENSION_MAX,
            "render dimensions must be <= 8192",
        )?;
        require(
            self.simulation.field_width >= SIMULATION_FIELD_DIMENSION_MIN
                && self.simulation.field_height >= SIMULATION_FIELD_DIMENSION_MIN,
            "simulation field dimensions must be at least 16",
        )?;
        require(
            self.simulation.substeps >= SIMULATION_SUBSTEPS_MIN,
            "simulation.substeps must be positive",
        )?;
        range(
            self.simulation.motion_scale,
            MOTION_SCALE_MIN,
            MOTION_SCALE_MAX,
            "simulation.motion_scale",
        )?;
        range(
            self.simulation.velocity_decay,
            UNIT_INTERVAL_MIN,
            UNIT_INTERVAL_MAX,
            "simulation.velocity_decay",
        )?;
        range(
            self.simulation.vorticity,
            SIMULATION_FORCE_MIN,
            SIMULATION_FORCE_MAX,
            "simulation.vorticity",
        )?;
        range(
            self.simulation.buoyancy,
            SIMULATION_FORCE_MIN,
            SIMULATION_FORCE_MAX,
            "simulation.buoyancy",
        )?;
        range(
            self.simulation.turbulence,
            SIMULATION_FORCE_MIN,
            SIMULATION_FORCE_MAX,
            "simulation.turbulence",
        )?;
        range(
            self.simulation.swirl,
            SIMULATION_FORCE_MIN,
            SIMULATION_FORCE_MAX,
            "simulation.swirl",
        )?;
        range(
            self.simulation.advection_dissipation,
            UNIT_INTERVAL_MIN,
            UNIT_INTERVAL_MAX,
            "simulation.advection_dissipation",
        )?;
        range(
            self.plasma.emission_strength,
            PLASMA_EMISSION_MIN,
            PLASMA_EMISSION_MAX,
            "plasma.emission_strength",
        )?;
        range(
            self.plasma.filament_density,
            PLASMA_FILAMENT_DENSITY_MIN,
            PLASMA_FILAMENT_DENSITY_MAX,
            "plasma.filament_density",
        )?;
        range(
            self.plasma.core_temperature,
            PLASMA_CORE_TEMPERATURE_MIN,
            PLASMA_CORE_TEMPERATURE_MAX,
            "plasma.core_temperature",
        )?;
        range(
            self.plasma.edge_wrapping,
            PLASMA_EDGE_WRAPPING_MIN,
            PLASMA_EDGE_WRAPPING_MAX,
            "plasma.edge_wrapping",
        )?;
        parse_hex_color(&self.plasma.color_core)?;
        parse_hex_color(&self.plasma.color_edge)?;
        require(
            self.particles.count <= PARTICLES_COUNT_MAX,
            "particles.count is too high",
        )?;
        range(
            self.particles.size_min,
            PARTICLES_SIZE_MIN_MIN,
            PARTICLES_SIZE_MIN_MAX,
            "particles.size_min",
        )?;
        range(
            self.particles.size_max,
            self.particles.size_min,
            PARTICLES_SIZE_MAX_MAX,
            "particles.size_max",
        )?;
        range(
            self.particles.lifetime,
            PARTICLES_LIFETIME_MIN,
            PARTICLES_LIFETIME_MAX,
            "particles.lifetime",
        )?;
        range(
            self.particles.flow_coupling,
            UNIT_INTERVAL_MIN,
            UNIT_INTERVAL_MAX,
            "particles.flow_coupling",
        )?;
        range(
            self.particles.ember_brightness,
            PARTICLES_EMBER_BRIGHTNESS_MIN,
            PARTICLES_EMBER_BRIGHTNESS_MAX,
            "particles.ember_brightness",
        )?;
        range(
            self.particles.gravity,
            PARTICLES_GRAVITY_MIN,
            PARTICLES_GRAVITY_MAX,
            "particles.gravity",
        )?;
        range(
            self.particles.drag,
            PARTICLES_DRAG_MIN,
            PARTICLES_DRAG_MAX,
            "particles.drag",
        )?;
        range(
            self.particles.thermal_lift,
            PARTICLES_THERMAL_LIFT_MIN,
            PARTICLES_THERMAL_LIFT_MAX,
            "particles.thermal_lift",
        )?;
        range(
            self.particles.jitter,
            PARTICLES_JITTER_MIN,
            PARTICLES_JITTER_MAX,
            "particles.jitter",
        )?;
        range(self.smoke.density, SMOKE_MIN, SMOKE_MAX, "smoke.density")?;
        range(
            self.smoke.scattering,
            SMOKE_MIN,
            SMOKE_MAX,
            "smoke.scattering",
        )?;
        range(
            self.smoke.absorption,
            SMOKE_MIN,
            SMOKE_MAX,
            "smoke.absorption",
        )?;
        parse_hex_color(&self.smoke.color_cool)?;
        parse_hex_color(&self.smoke.color_warm)?;
        range(
            self.bloom.threshold,
            BLOOM_THRESHOLD_MIN,
            BLOOM_THRESHOLD_MAX,
            "bloom.threshold",
        )?;
        range(
            self.bloom.strength,
            BLOOM_STRENGTH_MIN,
            BLOOM_STRENGTH_MAX,
            "bloom.strength",
        )?;
        range(
            self.bloom.radius,
            BLOOM_RADIUS_MIN,
            BLOOM_RADIUS_MAX,
            "bloom.radius",
        )?;
        require(
            (BLOOM_PYRAMID_LEVELS_MIN..=BLOOM_PYRAMID_LEVELS_MAX)
                .contains(&self.bloom.pyramid_levels),
            "bloom.pyramid_levels must be 1..=8",
        )?;
        range(
            self.grade.exposure,
            GRADE_EXPOSURE_MIN,
            GRADE_EXPOSURE_MAX,
            "grade.exposure",
        )?;
        range(
            self.grade.contrast,
            GRADE_CONTRAST_MIN,
            GRADE_CONTRAST_MAX,
            "grade.contrast",
        )?;
        range(
            self.grade.saturation,
            GRADE_SATURATION_MIN,
            GRADE_SATURATION_MAX,
            "grade.saturation",
        )?;
        range(
            self.grade.black_point,
            GRADE_BLACK_POINT_MIN,
            GRADE_BLACK_POINT_MAX,
            "grade.black_point",
        )?;
        range(
            self.grade.vignette,
            GRADE_VIGNETTE_MIN,
            GRADE_VIGNETTE_MAX,
            "grade.vignette",
        )?;
        range(
            self.grade.grain,
            GRADE_GRAIN_MIN,
            GRADE_GRAIN_MAX,
            "grade.grain",
        )?;
        require(
            !self.export.output_dir.trim().is_empty(),
            "export.output_dir must not be empty",
        )?;
        require(
            self.export.crf <= EXPORT_CRF_MAX,
            "export.crf must be <= 51",
        )?;
        if self.export.sequence_format == SequenceFormat::Exr {
            return Err(ConfigError::Invalid(
                "EXR sequence export is declared for future HDR output but is not implemented in v1"
                    .to_string(),
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

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::*;

    #[test]
    fn loads_valid_config() {
        let config: Config =
            serde_yaml::from_str(include_str!("../../../configs/inferno.production.yaml"))
                .expect("sample config parses");
        config.validate().expect("sample config validates");
    }

    #[test]
    fn loads_all_repo_configs() {
        let config_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../configs");
        for entry in fs::read_dir(config_dir).expect("config dir exists") {
            let path = entry.expect("config entry").path();
            let is_yaml = path
                .extension()
                .is_some_and(|extension| extension == "yaml" || extension == "yml");
            if !is_yaml {
                continue;
            }
            let config = Config::load(&path).unwrap_or_else(|err| {
                panic!("{} should load: {err}", path.display());
            });
            config
                .validate()
                .unwrap_or_else(|err| panic!("{} should validate: {err}", path.display()));
        }
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
