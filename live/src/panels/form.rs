use std::ops::RangeInclusive;

use eframe::egui::{self, DragValue, Slider, Ui};

const LABEL_WIDTH: f32 = 126.0;
const CONTROL_WIDTH: f32 = 174.0;
const DRAG_WIDTH: f32 = 104.0;
const ROW_HEIGHT: f32 = 22.0;

pub(super) fn configure_sidebar(ui: &mut Ui) {
    ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);
    ui.spacing_mut().button_padding = egui::vec2(8.0, 4.0);
    ui.spacing_mut().slider_width = CONTROL_WIDTH;
    ui.spacing_mut().combo_width = CONTROL_WIDTH;
}

pub(super) fn section(
    ui: &mut Ui,
    title: &str,
    default_open: bool,
    add_contents: impl FnOnce(&mut Ui) -> bool,
) -> bool {
    let mut changed = false;
    egui::CollapsingHeader::new(title)
        .default_open(default_open)
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 6.0);
            ui.spacing_mut().slider_width = CONTROL_WIDTH;
            ui.spacing_mut().combo_width = CONTROL_WIDTH;
            ui.add_space(2.0);
            changed |= add_contents(ui);
        });
    ui.add_space(6.0);
    changed
}

pub(super) fn subheading(ui: &mut Ui, label: &str) {
    ui.add_space(2.0);
    ui.strong(label);
    ui.add_space(2.0);
}

pub(super) fn row(ui: &mut Ui, label: &str, add_control: impl FnOnce(&mut Ui) -> bool) -> bool {
    ui.horizontal(|ui| {
        ui.add_sized([LABEL_WIDTH, ROW_HEIGHT], egui::Label::new(label));
        add_control(ui)
    })
    .inner
}

pub(super) fn text_row(ui: &mut Ui, label: &str, value: &mut String) -> bool {
    row(ui, label, |ui| {
        ui.add_sized(
            [CONTROL_WIDTH, ROW_HEIGHT],
            egui::TextEdit::singleline(value),
        )
        .changed()
    })
}

pub(super) fn slider_f32(
    ui: &mut Ui,
    label: &str,
    value: &mut f32,
    range: RangeInclusive<f32>,
) -> bool {
    row(ui, label, |ui| {
        ui.add_sized([CONTROL_WIDTH, ROW_HEIGHT], Slider::new(value, range))
            .changed()
    })
}

pub(super) fn drag_u32(
    ui: &mut Ui,
    label: &str,
    value: &mut u32,
    range: RangeInclusive<u32>,
    speed: f64,
) -> bool {
    row(ui, label, |ui| {
        ui.add_sized(
            [DRAG_WIDTH, ROW_HEIGHT],
            DragValue::new(value).range(range).speed(speed),
        )
        .changed()
    })
}

pub(super) fn drag_u64(ui: &mut Ui, label: &str, value: &mut u64, speed: f64) -> bool {
    row(ui, label, |ui| {
        ui.add_sized([DRAG_WIDTH, ROW_HEIGHT], DragValue::new(value).speed(speed))
            .changed()
    })
}

pub(super) fn drag_u8(
    ui: &mut Ui,
    label: &str,
    value: &mut u8,
    range: RangeInclusive<u8>,
    speed: f64,
) -> bool {
    row(ui, label, |ui| {
        ui.add_sized(
            [DRAG_WIDTH, ROW_HEIGHT],
            DragValue::new(value).range(range).speed(speed),
        )
        .changed()
    })
}

pub(super) fn combo_row(ui: &mut Ui, label: &str, add_combo: impl FnOnce(&mut Ui) -> bool) -> bool {
    row(ui, label, add_combo)
}
