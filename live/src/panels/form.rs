use std::ops::RangeInclusive;

use eframe::egui::{self, DragValue, Pos2, RichText, Sense, Slider, Ui};

use crate::app::theme;

const LABEL_WIDTH: f32 = 118.0;
const CONTROL_MIN_WIDTH: f32 = 164.0;
const CONTROL_MAX_WIDTH: f32 = 238.0;
const ROW_GAP: f32 = 10.0;
const DRAG_WIDTH: f32 = 112.0;
const ROW_HEIGHT: f32 = theme::CONTROL_HEIGHT;

pub(super) fn configure_sidebar(ui: &mut Ui) {
    ui.spacing_mut().item_spacing = egui::vec2(8.0, 7.0);
    ui.spacing_mut().button_padding = egui::vec2(10.0, 5.0);
    ui.spacing_mut().interact_size.y = ROW_HEIGHT;
    ui.spacing_mut().slider_width = CONTROL_MAX_WIDTH;
    ui.spacing_mut().combo_width = CONTROL_MAX_WIDTH;
}

pub(super) fn section(
    ui: &mut Ui,
    title: &str,
    default_open: bool,
    add_contents: impl FnOnce(&mut Ui) -> bool,
) -> bool {
    let mut changed = false;
    ui.add_space(8.0);
    draw_section_divider(ui);
    ui.add_space(4.0);
    egui::CollapsingHeader::new(RichText::new(title).size(13.0).strong().color(theme::TEXT))
        .default_open(default_open)
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 6.0);
            ui.spacing_mut().slider_width = control_width(ui);
            ui.spacing_mut().combo_width = control_width(ui);
            ui.add_space(4.0);
            changed |= add_contents(ui);
        });
    ui.add_space(4.0);
    changed
}

pub(super) fn subheading(ui: &mut Ui, label: &str) {
    ui.add_space(3.0);
    ui.label(
        RichText::new(label)
            .size(12.0)
            .strong()
            .color(theme::TEXT_MUTED),
    );
    ui.add_space(3.0);
}

pub(super) fn row(ui: &mut Ui, label: &str, add_control: impl FnOnce(&mut Ui) -> bool) -> bool {
    let control_width = width_for_row(ui.available_width());
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = ROW_GAP;
        ui.spacing_mut().slider_width = control_width;
        ui.spacing_mut().combo_width = control_width;
        ui.spacing_mut().interact_size.y = ROW_HEIGHT;
        ui.add_sized(
            [LABEL_WIDTH, ROW_HEIGHT],
            egui::Label::new(RichText::new(label).size(12.0).color(theme::TEXT_MUTED)).truncate(),
        );
        add_control(ui)
    })
    .inner
}

pub(super) fn text_row(ui: &mut Ui, label: &str, value: &mut String) -> bool {
    row(ui, label, |ui| {
        ui.add_sized(
            [control_width(ui), ROW_HEIGHT],
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
        ui.add_sized([control_width(ui), ROW_HEIGHT], Slider::new(value, range))
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

pub(super) fn control_width(ui: &Ui) -> f32 {
    ui.available_width()
        .clamp(CONTROL_MIN_WIDTH, CONTROL_MAX_WIDTH)
}

fn width_for_row(row_width: f32) -> f32 {
    (row_width - LABEL_WIDTH - ROW_GAP).clamp(CONTROL_MIN_WIDTH, CONTROL_MAX_WIDTH)
}

fn draw_section_divider(ui: &mut Ui) {
    let width = ui.available_width().max(1.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 1.0), Sense::hover());
    ui.painter().line_segment(
        [
            Pos2::new(rect.left(), rect.center().y),
            Pos2::new(rect.right(), rect.center().y),
        ],
        (1.0, theme::BORDER_SOFT),
    );
}
