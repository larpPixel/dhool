use dhool_config::{Config, EXPORT_CRF_MAX};
use eframe::egui::{self, DragValue, Ui};

use crate::panels::controls::{export_mode_combo, sequence_format_combo};

pub(super) fn export_panel(ui: &mut Ui, config: &mut Config) -> bool {
    let mut changed = false;
    egui::CollapsingHeader::new("Export").show(ui, |ui| {
        changed |= export_mode_combo(ui, "mode", &mut config.export.mode);
        changed |= sequence_format_combo(ui, "sequence format", &mut config.export.sequence_format);
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
        ui.label("output dir");
        changed |= ui
            .text_edit_singleline(&mut config.export.output_dir)
            .changed();
    });
    changed
}
