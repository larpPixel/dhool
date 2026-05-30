use std::time::{SystemTime, UNIX_EPOCH};

use dhool_config::{
    BLOOM_PYRAMID_LEVELS_MAX, BLOOM_PYRAMID_LEVELS_MIN, BLOOM_RADIUS_MAX, BLOOM_RADIUS_MIN,
    BLOOM_STRENGTH_MAX, BLOOM_STRENGTH_MIN, BLOOM_THRESHOLD_MAX, BLOOM_THRESHOLD_MIN, Config,
    GRADE_BLACK_POINT_MAX, GRADE_BLACK_POINT_MIN, GRADE_CONTRAST_MAX, GRADE_CONTRAST_MIN,
    GRADE_EXPOSURE_MAX, GRADE_EXPOSURE_MIN, GRADE_GRAIN_MAX, GRADE_GRAIN_MIN, GRADE_SATURATION_MAX,
    GRADE_SATURATION_MIN, GRADE_VIGNETTE_MAX, GRADE_VIGNETTE_MIN, MOTION_SCALE_MAX,
    MOTION_SCALE_MIN, PARTICLES_COUNT_MAX, PARTICLES_DRAG_MAX, PARTICLES_DRAG_MIN,
    PARTICLES_EMBER_BRIGHTNESS_MAX, PARTICLES_EMBER_BRIGHTNESS_MIN, PARTICLES_GRAVITY_MAX,
    PARTICLES_GRAVITY_MIN, PARTICLES_JITTER_MAX, PARTICLES_JITTER_MIN, PARTICLES_LIFETIME_MAX,
    PARTICLES_LIFETIME_MIN, PARTICLES_SIZE_MAX_MAX, PARTICLES_SIZE_MIN_MAX, PARTICLES_SIZE_MIN_MIN,
    PARTICLES_THERMAL_LIFT_MAX, PARTICLES_THERMAL_LIFT_MIN, PLASMA_CORE_TEMPERATURE_MAX,
    PLASMA_CORE_TEMPERATURE_MIN, PLASMA_EDGE_WRAPPING_MAX, PLASMA_EDGE_WRAPPING_MIN,
    PLASMA_EMISSION_MAX, PLASMA_EMISSION_MIN, PLASMA_FILAMENT_DENSITY_MAX,
    PLASMA_FILAMENT_DENSITY_MIN, RENDER_DIMENSION_MAX, SCENE_DURATION_MAX, SCENE_DURATION_MIN,
    SCENE_FPS_MAX, SCENE_FPS_MIN, SIMULATION_FIELD_DIMENSION_MIN, SIMULATION_FORCE_MAX,
    SIMULATION_FORCE_MIN, SIMULATION_SUBSTEPS_MIN, SIMULATION_SUBSTEPS_UI_MAX, SMOKE_MAX,
    SMOKE_MIN, UNIT_INTERVAL_MAX, UNIT_INTERVAL_MIN,
};
use dhool_core::splitmix64;

const PALETTES: &[(&str, &str, &str, &str)] = &[
    ("#ff431e", "#e66b1a", "#142f51", "#153f5c"),
    ("#ff1cd2", "#4444bb", "#17355c", "#6d2a16"),
    ("#ffd15a", "#e34b2f", "#0d2742", "#28516c"),
    ("#7dffcf", "#2f8df0", "#10243d", "#5d2639"),
    ("#ff7b34", "#ffc15a", "#101b30", "#55311f"),
    ("#b9f3ff", "#ff5a9c", "#162038", "#61402b"),
];

pub fn randomize_config(config: &mut Config) -> Result<(), String> {
    let seed = entropy_seed(config.scene.seed);
    apply_random_config(config, seed);
    config.validate().map_err(|err| err.to_string())
}

fn apply_random_config(config: &mut Config, seed: u64) {
    let mut rng = RandomSource::new(seed);
    let scene_seed = rng.next_u64();
    config.scene.name = format!("inferno_random_{:06x}", scene_seed & 0x00ff_ffff);
    config.scene.seed = scene_seed;
    config.scene.duration = rng.range_f32(8.0, 18.0, SCENE_DURATION_MIN, SCENE_DURATION_MAX);
    config.scene.fps = rng.pick_f32(&[24.0, 30.0, 60.0, 120.0], SCENE_FPS_MIN, SCENE_FPS_MAX);

    let field_scale = rng.range_f32(0.45, 0.72, 0.0, 1.0);
    config.simulation.field_width = scaled_dimension(
        config.render.width,
        field_scale,
        SIMULATION_FIELD_DIMENSION_MIN,
    );
    config.simulation.field_height = scaled_dimension(
        config.render.height,
        field_scale,
        SIMULATION_FIELD_DIMENSION_MIN,
    );
    config.simulation.substeps =
        rng.range_u32(1, 4, SIMULATION_SUBSTEPS_MIN, SIMULATION_SUBSTEPS_UI_MAX);
    config.simulation.motion_scale = rng.range_f32(0.45, 1.15, MOTION_SCALE_MIN, MOTION_SCALE_MAX);
    config.simulation.velocity_decay =
        rng.range_f32(0.985, 0.998, UNIT_INTERVAL_MIN, UNIT_INTERVAL_MAX);
    config.simulation.vorticity =
        rng.range_f32(0.35, 1.10, SIMULATION_FORCE_MIN, SIMULATION_FORCE_MAX);
    config.simulation.buoyancy =
        rng.range_f32(0.08, 0.36, SIMULATION_FORCE_MIN, SIMULATION_FORCE_MAX);
    config.simulation.turbulence =
        rng.range_f32(0.75, 2.25, SIMULATION_FORCE_MIN, SIMULATION_FORCE_MAX);
    config.simulation.swirl = rng.range_f32(0.40, 1.85, SIMULATION_FORCE_MIN, SIMULATION_FORCE_MAX);
    config.simulation.advection_dissipation =
        rng.range_f32(0.975, 0.995, UNIT_INTERVAL_MIN, UNIT_INTERVAL_MAX);

    let palette = PALETTES[rng.index(PALETTES.len())];
    config.plasma.emission_strength =
        rng.range_f32(2.8, 6.8, PLASMA_EMISSION_MIN, PLASMA_EMISSION_MAX);
    config.plasma.filament_density = rng.range_f32(
        1.0,
        2.8,
        PLASMA_FILAMENT_DENSITY_MIN,
        PLASMA_FILAMENT_DENSITY_MAX,
    );
    config.plasma.core_temperature = rng.range_f32(
        0.75,
        1.45,
        PLASMA_CORE_TEMPERATURE_MIN,
        PLASMA_CORE_TEMPERATURE_MAX,
    );
    config.plasma.edge_wrapping = rng.range_f32(
        0.55,
        1.25,
        PLASMA_EDGE_WRAPPING_MIN,
        PLASMA_EDGE_WRAPPING_MAX,
    );
    config.plasma.color_core = palette.0.to_string();
    config.plasma.color_edge = palette.1.to_string();

    config.particles.count = rng.range_u32(35_000, 140_000, 0, PARTICLES_COUNT_MAX);
    config.particles.size_min =
        rng.range_f32(0.04, 0.16, PARTICLES_SIZE_MIN_MIN, PARTICLES_SIZE_MIN_MAX);
    config.particles.size_max = rng.range_f32(
        config.particles.size_min + 0.35,
        config.particles.size_min + 1.40,
        config.particles.size_min,
        PARTICLES_SIZE_MAX_MAX,
    );
    config.particles.lifetime =
        rng.range_f32(4.0, 9.5, PARTICLES_LIFETIME_MIN, PARTICLES_LIFETIME_MAX);
    config.particles.flow_coupling =
        rng.range_f32(0.25, 0.72, UNIT_INTERVAL_MIN, UNIT_INTERVAL_MAX);
    config.particles.ember_brightness = rng.range_f32(
        0.80,
        1.85,
        PARTICLES_EMBER_BRIGHTNESS_MIN,
        PARTICLES_EMBER_BRIGHTNESS_MAX,
    );
    config.particles.gravity =
        rng.range_f32(0.05, 0.24, PARTICLES_GRAVITY_MIN, PARTICLES_GRAVITY_MAX);
    config.particles.drag = rng.range_f32(2.2, 5.2, PARTICLES_DRAG_MIN, PARTICLES_DRAG_MAX);
    config.particles.thermal_lift = rng.range_f32(
        0.40,
        1.25,
        PARTICLES_THERMAL_LIFT_MIN,
        PARTICLES_THERMAL_LIFT_MAX,
    );
    config.particles.jitter = rng.range_f32(0.05, 0.36, PARTICLES_JITTER_MIN, PARTICLES_JITTER_MAX);

    config.smoke.density = rng.range_f32(1.1, 2.8, SMOKE_MIN, SMOKE_MAX);
    config.smoke.scattering = rng.range_f32(0.80, 1.75, SMOKE_MIN, SMOKE_MAX);
    config.smoke.absorption = rng.range_f32(0.25, 0.75, SMOKE_MIN, SMOKE_MAX);
    config.smoke.color_cool = palette.2.to_string();
    config.smoke.color_warm = palette.3.to_string();

    config.bloom.threshold = rng.range_f32(0.55, 1.20, BLOOM_THRESHOLD_MIN, BLOOM_THRESHOLD_MAX);
    config.bloom.strength = rng.range_f32(0.55, 1.35, BLOOM_STRENGTH_MIN, BLOOM_STRENGTH_MAX);
    config.bloom.radius = rng.range_f32(0.75, 1.60, BLOOM_RADIUS_MIN, BLOOM_RADIUS_MAX);
    config.bloom.pyramid_levels =
        rng.range_u32(4, 6, BLOOM_PYRAMID_LEVELS_MIN, BLOOM_PYRAMID_LEVELS_MAX);

    config.grade.exposure = rng.range_f32(-0.40, 0.15, GRADE_EXPOSURE_MIN, GRADE_EXPOSURE_MAX);
    config.grade.contrast = rng.range_f32(1.05, 1.35, GRADE_CONTRAST_MIN, GRADE_CONTRAST_MAX);
    config.grade.saturation = rng.range_f32(0.55, 0.95, GRADE_SATURATION_MIN, GRADE_SATURATION_MAX);
    config.grade.black_point =
        rng.range_f32(0.012, 0.035, GRADE_BLACK_POINT_MIN, GRADE_BLACK_POINT_MAX);
    config.grade.vignette = rng.range_f32(0.45, 0.85, GRADE_VIGNETTE_MIN, GRADE_VIGNETTE_MAX);
    config.grade.grain = rng.range_f32(0.006, 0.025, GRADE_GRAIN_MIN, GRADE_GRAIN_MAX);
}

fn entropy_seed(current_seed: u64) -> u64 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(0);
    splitmix64(nanos ^ current_seed)
}

fn scaled_dimension(value: u32, scale: f32, min: u32) -> u32 {
    let scaled = ((value as f32 * scale).round() as u32).max(min);
    scaled.min(RENDER_DIMENSION_MAX)
}

struct RandomSource {
    seed: u64,
    index: u64,
}

impl RandomSource {
    fn new(seed: u64) -> Self {
        Self { seed, index: 0 }
    }

    fn next_u64(&mut self) -> u64 {
        self.index += 1;
        splitmix64(self.seed ^ self.index)
    }

    fn unit(&mut self) -> f32 {
        ((self.next_u64() >> 40) as f32) / ((1_u64 << 24) as f32)
    }

    fn index(&mut self, len: usize) -> usize {
        (self.next_u64() as usize) % len.max(1)
    }

    fn range_f32(&mut self, min: f32, max: f32, limit_min: f32, limit_max: f32) -> f32 {
        (min + (max - min) * self.unit()).clamp(limit_min, limit_max)
    }

    fn range_u32(&mut self, min: u32, max: u32, limit_min: u32, limit_max: u32) -> u32 {
        let value = min + (self.unit() * (max - min + 1) as f32).floor() as u32;
        value.clamp(limit_min, limit_max)
    }

    fn pick_f32(&mut self, values: &[f32], limit_min: f32, limit_max: f32) -> f32 {
        values[self.index(values.len())].clamp(limit_min, limit_max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn randomized_config_validates_and_preserves_output_path() {
        let mut config: Config =
            serde_yaml::from_str(include_str!("../../configs/inferno.production.yaml"))
                .expect("sample config parses");
        let output_dir = config.export.output_dir.clone();

        apply_random_config(&mut config, 42);

        config.validate().expect("random config validates");
        assert_eq!(config.export.output_dir, output_dir);
        assert_ne!(config.scene.seed, 546642);
    }
}
