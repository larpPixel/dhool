use dhool_config::{Config, RENDER_DIMENSION_MAX, RENDER_DIMENSION_MIN};
use eframe::egui::Ui;

use crate::panels::controls::{backend_combo, internal_format_combo, quality_combo};
use crate::panels::form;

pub(super) fn render_panel(
    ui: &mut Ui,
    config: &mut Config,
    preview_scale: &mut f32,
    preview_field_scale: &mut f32,
) -> bool {
    let mut changed = false;
    form::section(ui, "Preview", true, |ui| {
        changed |= form::slider_f32(ui, "Preview scale", preview_scale, 0.25..=1.0);
        changed |= form::slider_f32(ui, "Field scale", preview_field_scale, 0.25..=1.0);
        changed |= form::drag_u32(
            ui,
            "Width",
            &mut config.render.width,
            RENDER_DIMENSION_MIN..=RENDER_DIMENSION_MAX,
            16.0,
        );
        changed |= form::drag_u32(
            ui,
            "Height",
            &mut config.render.height,
            RENDER_DIMENSION_MIN..=RENDER_DIMENSION_MAX,
            16.0,
        );
        changed |= form::combo_row(ui, "Quality", |ui| {
            quality_combo(ui, &mut config.render.quality)
        });
        changed |= form::combo_row(ui, "Internal format", |ui| {
            internal_format_combo(ui, &mut config.render.internal_format)
        });
        changed
    })
}

pub(super) fn backend_panel(ui: &mut Ui, config: &mut Config) -> bool {
    let mut changed = false;
    form::section(ui, "Backend", false, |ui| {
        changed |= form::combo_row(ui, "Preferred", |ui| {
            backend_combo(ui, &mut config.backend.preferred)
        });
        changed |= form::row(ui, "Require GPU", |ui| {
            ui.checkbox(&mut config.backend.require_gpu, "").changed()
        });
        changed
    })
}
