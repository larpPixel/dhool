use dhool_config::{Config, SCENE_DURATION_MAX, SCENE_DURATION_MIN, SCENE_FPS_MAX, SCENE_FPS_MIN};
use eframe::egui::{self, DragValue, Ui};

pub(super) fn scene_panel(ui: &mut Ui, config: &mut Config) -> bool {
    let mut changed = false;
    egui::CollapsingHeader::new("Scene")
        .default_open(true)
        .show(ui, |ui| {
            ui.label("name");
            changed |= ui.text_edit_singleline(&mut config.scene.name).changed();
            changed |= ui
                .add(
                    DragValue::new(&mut config.scene.seed)
                        .speed(1.0)
                        .prefix("seed "),
                )
                .changed();
            changed |= ui
                .add(
                    egui::Slider::new(
                        &mut config.scene.duration,
                        SCENE_DURATION_MIN..=SCENE_DURATION_MAX,
                    )
                    .text("duration"),
                )
                .changed();
            changed |= ui
                .add(
                    egui::Slider::new(&mut config.scene.fps, SCENE_FPS_MIN..=SCENE_FPS_MAX)
                        .text("fps"),
                )
                .changed();
        });
    changed
}
