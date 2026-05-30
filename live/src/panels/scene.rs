use dhool_config::{Config, SCENE_DURATION_MAX, SCENE_DURATION_MIN, SCENE_FPS_MAX, SCENE_FPS_MIN};
use eframe::egui::Ui;

use crate::panels::form;

pub(super) fn scene_panel(ui: &mut Ui, config: &mut Config) -> bool {
    let mut changed = false;
    form::section(ui, "Scene", true, |ui| {
        changed |= form::text_row(ui, "Name", &mut config.scene.name);
        changed |= form::drag_u64(ui, "Seed", &mut config.scene.seed, 1.0);
        changed
    })
}

pub(super) fn export_timing_controls(ui: &mut Ui, config: &mut Config) -> bool {
    let mut changed = false;
    form::subheading(ui, "Animation");
    changed |= form::slider_f32(
        ui,
        "Duration",
        &mut config.scene.duration,
        SCENE_DURATION_MIN..=SCENE_DURATION_MAX,
    );
    changed |= form::slider_f32(
        ui,
        "FPS",
        &mut config.scene.fps,
        SCENE_FPS_MIN..=SCENE_FPS_MAX,
    );
    form::row(ui, "Frames", |ui| {
        ui.label(config.frame_count(None).to_string());
        false
    });
    changed
}
