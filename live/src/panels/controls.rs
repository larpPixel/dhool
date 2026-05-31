use dhool_config::{BackendKind, InternalFormat, Quality, parse_hex_color};
use eframe::egui::{self, Color32, Ui};

use crate::app::theme;
use crate::panels::form;

pub(super) fn quality_combo(ui: &mut Ui, value: &mut Quality) -> bool {
    let mut changed = false;
    egui::ComboBox::from_id_salt("render-quality")
        .selected_text(quality_label(*value))
        .width(ui.spacing().combo_width)
        .show_ui(ui, |ui| {
            changed |= ui
                .selectable_value(value, Quality::Preview, quality_label(Quality::Preview))
                .changed();
            changed |= ui
                .selectable_value(value, Quality::Final, quality_label(Quality::Final))
                .changed();
            changed |= ui
                .selectable_value(value, Quality::Ultra, quality_label(Quality::Ultra))
                .changed();
        });
    changed
}

pub(super) fn backend_combo(ui: &mut Ui, value: &mut BackendKind) -> bool {
    let mut changed = false;
    egui::ComboBox::from_id_salt("backend-kind")
        .selected_text(backend_label(*value))
        .width(ui.spacing().combo_width)
        .show_ui(ui, |ui| {
            changed |= ui
                .selectable_value(value, BackendKind::Cuda, backend_label(BackendKind::Cuda))
                .changed();
            changed |= ui
                .selectable_value(
                    value,
                    BackendKind::CpuReference,
                    backend_label(BackendKind::CpuReference),
                )
                .changed();
        });
    changed
}

pub(super) fn internal_format_combo(ui: &mut Ui, value: &mut InternalFormat) -> bool {
    let mut changed = false;
    egui::ComboBox::from_id_salt("internal-format")
        .selected_text(internal_format_label(*value))
        .width(ui.spacing().combo_width)
        .show_ui(ui, |ui| {
            changed |= ui
                .selectable_value(
                    value,
                    InternalFormat::Rgba16f,
                    internal_format_label(InternalFormat::Rgba16f),
                )
                .changed();
        });
    changed
}

pub(super) fn hex_color_control(ui: &mut Ui, label: &str, value: &mut String) -> bool {
    form::row(ui, label, |ui| {
        let mut changed = false;
        let mut color = hex_to_color32(value).unwrap_or(Color32::WHITE);
        ui.spacing_mut().interact_size = egui::vec2(theme::CONTROL_HEIGHT, theme::CONTROL_HEIGHT);
        if egui::color_picker::color_edit_button_srgba(
            ui,
            &mut color,
            egui::color_picker::Alpha::Opaque,
        )
        .changed()
        {
            *value = color32_to_hex(color);
            changed = true;
        }
        let text_width =
            (form::control_span_width(ui) - theme::CONTROL_HEIGHT - ui.spacing().item_spacing.x)
                .max(104.0);
        changed |= ui
            .add_sized(
                [text_width, theme::CONTROL_HEIGHT],
                egui::TextEdit::singleline(value),
            )
            .changed();
        changed
    })
}

fn hex_to_color32(value: &str) -> Option<Color32> {
    let rgb = parse_hex_color(value).ok()?;
    Some(Color32::from_rgb(
        channel_to_u8(rgb.r),
        channel_to_u8(rgb.g),
        channel_to_u8(rgb.b),
    ))
}

fn channel_to_u8(value: f32) -> u8 {
    (value * 255.0).round().clamp(0.0, 255.0) as u8
}

fn color32_to_hex(color: Color32) -> String {
    format!("#{:02x}{:02x}{:02x}", color.r(), color.g(), color.b())
}

fn quality_label(value: Quality) -> &'static str {
    match value {
        Quality::Preview => "preview",
        Quality::Final => "final",
        Quality::Ultra => "ultra",
    }
}

fn backend_label(value: BackendKind) -> &'static str {
    match value {
        BackendKind::Cuda => "cuda",
        BackendKind::CpuReference => "cpu-reference",
    }
}

fn internal_format_label(value: InternalFormat) -> &'static str {
    match value {
        InternalFormat::Rgba16f => "rgba16f",
    }
}
