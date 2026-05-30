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
use eframe::egui::{self, DragValue, Ui};

use crate::panels::controls::hex_color_control;

pub(super) fn plasma_panel(ui: &mut Ui, config: &mut Config) -> bool {
    let mut changed = false;
    egui::CollapsingHeader::new("Plasma")
        .default_open(true)
        .show(ui, |ui| {
            changed |= ui
                .add(
                    egui::Slider::new(
                        &mut config.plasma.emission_strength,
                        PLASMA_EMISSION_MIN..=PLASMA_EMISSION_MAX,
                    )
                    .text("emission strength"),
                )
                .changed();
            changed |= ui
                .add(
                    egui::Slider::new(
                        &mut config.plasma.filament_density,
                        PLASMA_FILAMENT_DENSITY_MIN..=PLASMA_FILAMENT_DENSITY_MAX,
                    )
                    .text("filament density"),
                )
                .changed();
            changed |= ui
                .add(
                    egui::Slider::new(
                        &mut config.plasma.core_temperature,
                        PLASMA_CORE_TEMPERATURE_MIN..=PLASMA_CORE_TEMPERATURE_MAX,
                    )
                    .text("core temperature"),
                )
                .changed();
            changed |= ui
                .add(
                    egui::Slider::new(
                        &mut config.plasma.edge_wrapping,
                        PLASMA_EDGE_WRAPPING_MIN..=PLASMA_EDGE_WRAPPING_MAX,
                    )
                    .text("edge wrapping"),
                )
                .changed();
            changed |= hex_color_control(ui, "color core", &mut config.plasma.color_core);
            changed |= hex_color_control(ui, "color edge", &mut config.plasma.color_edge);
        });
    changed
}

pub(super) fn particles_panel(ui: &mut Ui, config: &mut Config) -> bool {
    let mut changed = false;
    egui::CollapsingHeader::new("Particles").show(ui, |ui| {
        changed |= ui
            .add(
                DragValue::new(&mut config.particles.count)
                    .range(0..=PARTICLES_COUNT_MAX)
                    .speed(100.0)
                    .prefix("count "),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(
                    &mut config.particles.size_min,
                    PARTICLES_SIZE_MIN_MIN..=PARTICLES_SIZE_MIN_MAX,
                )
                .text("size min"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(
                    &mut config.particles.size_max,
                    PARTICLES_SIZE_MIN_MIN..=PARTICLES_SIZE_MAX_MAX,
                )
                .text("size max"),
            )
            .changed();
        if config.particles.size_max < config.particles.size_min {
            config.particles.size_max = config.particles.size_min;
            changed = true;
        }
        changed |= ui
            .add(
                egui::Slider::new(
                    &mut config.particles.lifetime,
                    PARTICLES_LIFETIME_MIN..=PARTICLES_LIFETIME_MAX,
                )
                .text("lifetime"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(
                    &mut config.particles.flow_coupling,
                    UNIT_INTERVAL_MIN..=UNIT_INTERVAL_MAX,
                )
                .text("flow coupling"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(
                    &mut config.particles.ember_brightness,
                    PARTICLES_EMBER_BRIGHTNESS_MIN..=PARTICLES_EMBER_BRIGHTNESS_MAX,
                )
                .text("ember brightness"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(
                    &mut config.particles.gravity,
                    PARTICLES_GRAVITY_MIN..=PARTICLES_GRAVITY_MAX,
                )
                .text("gravity"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(
                    &mut config.particles.drag,
                    PARTICLES_DRAG_MIN..=PARTICLES_DRAG_MAX,
                )
                .text("drag"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(
                    &mut config.particles.thermal_lift,
                    PARTICLES_THERMAL_LIFT_MIN..=PARTICLES_THERMAL_LIFT_MAX,
                )
                .text("thermal lift"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(
                    &mut config.particles.jitter,
                    PARTICLES_JITTER_MIN..=PARTICLES_JITTER_MAX,
                )
                .text("jitter"),
            )
            .changed();
    });
    changed
}

pub(super) fn smoke_panel(ui: &mut Ui, config: &mut Config) -> bool {
    let mut changed = false;
    egui::CollapsingHeader::new("Smoke").show(ui, |ui| {
        changed |= ui
            .add(
                egui::Slider::new(&mut config.smoke.density, SMOKE_MIN..=SMOKE_MAX).text("density"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(&mut config.smoke.scattering, SMOKE_MIN..=SMOKE_MAX)
                    .text("scattering"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(&mut config.smoke.absorption, SMOKE_MIN..=SMOKE_MAX)
                    .text("absorption"),
            )
            .changed();
        changed |= hex_color_control(ui, "color cool", &mut config.smoke.color_cool);
        changed |= hex_color_control(ui, "color warm", &mut config.smoke.color_warm);
    });
    changed
}

pub(super) fn bloom_panel(ui: &mut Ui, config: &mut Config) -> bool {
    let mut changed = false;
    egui::CollapsingHeader::new("Bloom").show(ui, |ui| {
        changed |= ui
            .add(
                egui::Slider::new(
                    &mut config.bloom.threshold,
                    BLOOM_THRESHOLD_MIN..=BLOOM_THRESHOLD_MAX,
                )
                .text("threshold"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(
                    &mut config.bloom.strength,
                    BLOOM_STRENGTH_MIN..=BLOOM_STRENGTH_MAX,
                )
                .text("strength"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(
                    &mut config.bloom.radius,
                    BLOOM_RADIUS_MIN..=BLOOM_RADIUS_MAX,
                )
                .text("radius"),
            )
            .changed();
        changed |= ui
            .add(
                DragValue::new(&mut config.bloom.pyramid_levels)
                    .range(BLOOM_PYRAMID_LEVELS_MIN..=BLOOM_PYRAMID_LEVELS_MAX)
                    .speed(1.0)
                    .prefix("pyramid levels "),
            )
            .changed();
    });
    changed
}

pub(super) fn grade_panel(ui: &mut Ui, config: &mut Config) -> bool {
    let mut changed = false;
    egui::CollapsingHeader::new("Grade").show(ui, |ui| {
        changed |= ui
            .add(
                egui::Slider::new(
                    &mut config.grade.exposure,
                    GRADE_EXPOSURE_MIN..=GRADE_EXPOSURE_MAX,
                )
                .text("exposure"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(
                    &mut config.grade.contrast,
                    GRADE_CONTRAST_MIN..=GRADE_CONTRAST_MAX,
                )
                .text("contrast"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(
                    &mut config.grade.saturation,
                    GRADE_SATURATION_MIN..=GRADE_SATURATION_MAX,
                )
                .text("saturation"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(
                    &mut config.grade.black_point,
                    GRADE_BLACK_POINT_MIN..=GRADE_BLACK_POINT_MAX,
                )
                .text("black point"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(
                    &mut config.grade.vignette,
                    GRADE_VIGNETTE_MIN..=GRADE_VIGNETTE_MAX,
                )
                .text("vignette"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(&mut config.grade.grain, GRADE_GRAIN_MIN..=GRADE_GRAIN_MAX)
                    .text("grain"),
            )
            .changed();
    });
    changed
}
