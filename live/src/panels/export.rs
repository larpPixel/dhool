use dhool_config::{Config, EXPORT_CRF_MAX};
use eframe::egui::Ui;

use crate::panels::form;

pub(super) fn video_settings(ui: &mut Ui, config: &mut Config) -> bool {
    let mut changed = false;
    form::subheading(ui, "Video");
    changed |= form::text_row(ui, "Codec", &mut config.export.video_codec);
    changed |= form::drag_u8(ui, "CRF", &mut config.export.crf, 0..=EXPORT_CRF_MAX, 1.0);
    changed |= form::text_row(ui, "Preset", &mut config.export.preset);
    changed
}
