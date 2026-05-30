use dhool_config::{Config, RENDER_DIMENSION_MAX, RENDER_DIMENSION_MIN};
use eframe::egui::{self, DragValue, Ui};

use crate::panels::controls::{backend_combo, internal_format_combo, quality_combo};

pub(super) fn render_panel(
    ui: &mut Ui,
    config: &mut Config,
    preview_scale: &mut f32,
    preview_field_scale: &mut f32,
) -> bool {
    let mut changed = false;
    egui::CollapsingHeader::new("Render")
        .default_open(true)
        .show(ui, |ui| {
            changed |= ui
                .add(egui::Slider::new(preview_scale, 0.25..=1.0).text("preview scale"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(preview_field_scale, 0.25..=1.0).text("field scale"))
                .changed();
            changed |= ui
                .add(
                    DragValue::new(&mut config.render.width)
                        .range(RENDER_DIMENSION_MIN..=RENDER_DIMENSION_MAX)
                        .speed(16.0)
                        .prefix("width "),
                )
                .changed();
            changed |= ui
                .add(
                    DragValue::new(&mut config.render.height)
                        .range(RENDER_DIMENSION_MIN..=RENDER_DIMENSION_MAX)
                        .speed(16.0)
                        .prefix("height "),
                )
                .changed();
            changed |= quality_combo(ui, "quality", &mut config.render.quality);
            changed |= internal_format_combo(ui, &mut config.render.internal_format);
        });
    changed
}

pub(super) fn backend_panel(ui: &mut Ui, config: &mut Config) -> bool {
    let mut changed = false;
    egui::CollapsingHeader::new("Backend").show(ui, |ui| {
        changed |= backend_combo(ui, "preferred", &mut config.backend.preferred);
        changed |= ui
            .checkbox(&mut config.backend.require_gpu, "require gpu")
            .changed();
    });
    changed
}
