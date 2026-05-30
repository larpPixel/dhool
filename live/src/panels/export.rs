use dhool_config::{Config, EXPORT_CRF_MAX};
use eframe::egui::{DragValue, Ui};

pub(super) fn video_settings(ui: &mut Ui, config: &mut Config) -> bool {
    let mut changed = false;
    ui.strong("Video");
    ui.label("video codec");
    changed |= ui
        .text_edit_singleline(&mut config.export.video_codec)
        .changed();
    changed |= ui
        .add(
            DragValue::new(&mut config.export.crf)
                .range(0..=EXPORT_CRF_MAX)
                .speed(1.0)
                .prefix("crf "),
        )
        .changed();
    ui.label("preset");
    changed |= ui.text_edit_singleline(&mut config.export.preset).changed();
    changed
}
