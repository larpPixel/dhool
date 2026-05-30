use dhool_config::{
    BLOOM_PYRAMID_LEVELS_MAX, BLOOM_PYRAMID_LEVELS_MIN, BLOOM_RADIUS_MAX, BLOOM_RADIUS_MIN,
    BLOOM_STRENGTH_MAX, BLOOM_STRENGTH_MIN, BLOOM_THRESHOLD_MAX, BLOOM_THRESHOLD_MIN, Config,
    GRADE_BLACK_POINT_MAX, GRADE_BLACK_POINT_MIN, GRADE_CONTRAST_MAX, GRADE_CONTRAST_MIN,
    GRADE_EXPOSURE_MAX, GRADE_EXPOSURE_MIN, GRADE_GRAIN_MAX, GRADE_GRAIN_MIN, GRADE_SATURATION_MAX,
    GRADE_SATURATION_MIN, GRADE_VIGNETTE_MAX, GRADE_VIGNETTE_MIN, PARTICLES_COUNT_MAX,
    PARTICLES_DRAG_MAX, PARTICLES_DRAG_MIN, PARTICLES_EMBER_BRIGHTNESS_MAX,
    PARTICLES_EMBER_BRIGHTNESS_MIN, PARTICLES_GRAVITY_MAX, PARTICLES_GRAVITY_MIN,
    PARTICLES_JITTER_MAX, PARTICLES_JITTER_MIN, PARTICLES_LIFETIME_MAX, PARTICLES_LIFETIME_MIN,
    PARTICLES_SIZE_MAX_MAX, PARTICLES_SIZE_MIN_MAX, PARTICLES_SIZE_MIN_MIN,
    PARTICLES_THERMAL_LIFT_MAX, PARTICLES_THERMAL_LIFT_MIN, PLASMA_CORE_TEMPERATURE_MAX,
    PLASMA_CORE_TEMPERATURE_MIN, PLASMA_EDGE_WRAPPING_MAX, PLASMA_EDGE_WRAPPING_MIN,
    PLASMA_EMISSION_MAX, PLASMA_EMISSION_MIN, PLASMA_FILAMENT_DENSITY_MAX,
    PLASMA_FILAMENT_DENSITY_MIN, SMOKE_MAX, SMOKE_MIN, UNIT_INTERVAL_MAX, UNIT_INTERVAL_MIN,
};
use eframe::egui::Ui;

use crate::panels::controls::hex_color_control;
use crate::panels::form;

pub(super) fn plasma_panel(ui: &mut Ui, config: &mut Config) -> bool {
    let mut changed = false;
    form::section(ui, "Plasma", true, |ui| {
        changed |= form::slider_f32(
            ui,
            "Emission",
            &mut config.plasma.emission_strength,
            PLASMA_EMISSION_MIN..=PLASMA_EMISSION_MAX,
        );
        changed |= form::slider_f32(
            ui,
            "Filaments",
            &mut config.plasma.filament_density,
            PLASMA_FILAMENT_DENSITY_MIN..=PLASMA_FILAMENT_DENSITY_MAX,
        );
        changed |= form::slider_f32(
            ui,
            "Temperature",
            &mut config.plasma.core_temperature,
            PLASMA_CORE_TEMPERATURE_MIN..=PLASMA_CORE_TEMPERATURE_MAX,
        );
        changed |= form::slider_f32(
            ui,
            "Edge wrap",
            &mut config.plasma.edge_wrapping,
            PLASMA_EDGE_WRAPPING_MIN..=PLASMA_EDGE_WRAPPING_MAX,
        );
        changed |= hex_color_control(ui, "Core color", &mut config.plasma.color_core);
        changed |= hex_color_control(ui, "Edge color", &mut config.plasma.color_edge);
        changed
    })
}

pub(super) fn particles_panel(ui: &mut Ui, config: &mut Config) -> bool {
    let mut changed = false;
    form::section(ui, "Particles", false, |ui| {
        changed |= form::drag_u32(
            ui,
            "Count",
            &mut config.particles.count,
            0..=PARTICLES_COUNT_MAX,
            100.0,
        );
        changed |= form::slider_f32(
            ui,
            "Min size",
            &mut config.particles.size_min,
            PARTICLES_SIZE_MIN_MIN..=PARTICLES_SIZE_MIN_MAX,
        );
        changed |= form::slider_f32(
            ui,
            "Max size",
            &mut config.particles.size_max,
            PARTICLES_SIZE_MIN_MIN..=PARTICLES_SIZE_MAX_MAX,
        );
        if config.particles.size_max < config.particles.size_min {
            config.particles.size_max = config.particles.size_min;
            changed = true;
        }
        changed |= form::slider_f32(
            ui,
            "Lifetime",
            &mut config.particles.lifetime,
            PARTICLES_LIFETIME_MIN..=PARTICLES_LIFETIME_MAX,
        );
        changed |= form::slider_f32(
            ui,
            "Flow coupling",
            &mut config.particles.flow_coupling,
            UNIT_INTERVAL_MIN..=UNIT_INTERVAL_MAX,
        );
        changed |= form::slider_f32(
            ui,
            "Brightness",
            &mut config.particles.ember_brightness,
            PARTICLES_EMBER_BRIGHTNESS_MIN..=PARTICLES_EMBER_BRIGHTNESS_MAX,
        );
        changed |= form::slider_f32(
            ui,
            "Gravity",
            &mut config.particles.gravity,
            PARTICLES_GRAVITY_MIN..=PARTICLES_GRAVITY_MAX,
        );
        changed |= form::slider_f32(
            ui,
            "Drag",
            &mut config.particles.drag,
            PARTICLES_DRAG_MIN..=PARTICLES_DRAG_MAX,
        );
        changed |= form::slider_f32(
            ui,
            "Thermal lift",
            &mut config.particles.thermal_lift,
            PARTICLES_THERMAL_LIFT_MIN..=PARTICLES_THERMAL_LIFT_MAX,
        );
        changed |= form::slider_f32(
            ui,
            "Jitter",
            &mut config.particles.jitter,
            PARTICLES_JITTER_MIN..=PARTICLES_JITTER_MAX,
        );
        changed
    })
}

pub(super) fn smoke_panel(ui: &mut Ui, config: &mut Config) -> bool {
    let mut changed = false;
    form::section(ui, "Smoke", false, |ui| {
        changed |= form::slider_f32(
            ui,
            "Density",
            &mut config.smoke.density,
            SMOKE_MIN..=SMOKE_MAX,
        );
        changed |= form::slider_f32(
            ui,
            "Scattering",
            &mut config.smoke.scattering,
            SMOKE_MIN..=SMOKE_MAX,
        );
        changed |= form::slider_f32(
            ui,
            "Absorption",
            &mut config.smoke.absorption,
            SMOKE_MIN..=SMOKE_MAX,
        );
        changed |= hex_color_control(ui, "Cool color", &mut config.smoke.color_cool);
        changed |= hex_color_control(ui, "Warm color", &mut config.smoke.color_warm);
        changed
    })
}

pub(super) fn bloom_panel(ui: &mut Ui, config: &mut Config) -> bool {
    let mut changed = false;
    form::section(ui, "Bloom", false, |ui| {
        changed |= form::slider_f32(
            ui,
            "Threshold",
            &mut config.bloom.threshold,
            BLOOM_THRESHOLD_MIN..=BLOOM_THRESHOLD_MAX,
        );
        changed |= form::slider_f32(
            ui,
            "Strength",
            &mut config.bloom.strength,
            BLOOM_STRENGTH_MIN..=BLOOM_STRENGTH_MAX,
        );
        changed |= form::slider_f32(
            ui,
            "Radius",
            &mut config.bloom.radius,
            BLOOM_RADIUS_MIN..=BLOOM_RADIUS_MAX,
        );
        changed |= form::drag_u32(
            ui,
            "Levels",
            &mut config.bloom.pyramid_levels,
            BLOOM_PYRAMID_LEVELS_MIN..=BLOOM_PYRAMID_LEVELS_MAX,
            1.0,
        );
        changed
    })
}

pub(super) fn grade_panel(ui: &mut Ui, config: &mut Config) -> bool {
    let mut changed = false;
    form::section(ui, "Grade", false, |ui| {
        changed |= form::slider_f32(
            ui,
            "Exposure",
            &mut config.grade.exposure,
            GRADE_EXPOSURE_MIN..=GRADE_EXPOSURE_MAX,
        );
        changed |= form::slider_f32(
            ui,
            "Contrast",
            &mut config.grade.contrast,
            GRADE_CONTRAST_MIN..=GRADE_CONTRAST_MAX,
        );
        changed |= form::slider_f32(
            ui,
            "Saturation",
            &mut config.grade.saturation,
            GRADE_SATURATION_MIN..=GRADE_SATURATION_MAX,
        );
        changed |= form::slider_f32(
            ui,
            "Black point",
            &mut config.grade.black_point,
            GRADE_BLACK_POINT_MIN..=GRADE_BLACK_POINT_MAX,
        );
        changed |= form::slider_f32(
            ui,
            "Vignette",
            &mut config.grade.vignette,
            GRADE_VIGNETTE_MIN..=GRADE_VIGNETTE_MAX,
        );
        changed |= form::slider_f32(
            ui,
            "Grain",
            &mut config.grade.grain,
            GRADE_GRAIN_MIN..=GRADE_GRAIN_MAX,
        );
        changed
    })
}
