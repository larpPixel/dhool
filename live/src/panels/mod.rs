mod controls;
mod effects;
mod export;
mod render;
mod scene;
mod simulation;

use dhool_config::Config;
use eframe::egui::Ui;

pub fn draw_all(
    ui: &mut Ui,
    config: &mut Config,
    preview_scale: &mut f32,
    preview_field_scale: &mut f32,
) -> bool {
    ui.spacing_mut().slider_width = 180.0;

    let mut changed = false;
    changed |= scene::scene_panel(ui, config);
    changed |= render::render_panel(ui, config, preview_scale, preview_field_scale);
    changed |= render::backend_panel(ui, config);
    changed |= simulation::simulation_panel(ui, config);
    changed |= effects::plasma_panel(ui, config);
    changed |= effects::particles_panel(ui, config);
    changed |= effects::smoke_panel(ui, config);
    changed |= effects::bloom_panel(ui, config);
    changed |= effects::grade_panel(ui, config);
    changed |= export::export_panel(ui, config);
    changed
}
