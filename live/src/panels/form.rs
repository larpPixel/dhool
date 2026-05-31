use std::ops::RangeInclusive;

use eframe::egui::{
    self, Align, Align2, DragValue, FontId, Layout, Pos2, Rect, RichText, Sense, Slider, Ui, Vec2,
};

use crate::app::theme;

const LABEL_WIDTH: f32 = 122.0;
const CONTROL_MIN_WIDTH: f32 = 142.0;
const CONTROL_MAX_WIDTH: f32 = 250.0;
const VALUE_WIDTH: f32 = 56.0;
const COLUMN_GAP: f32 = 14.0;
const VALUE_GAP: f32 = 10.0;
const CONTROL_INNER_GAP: f32 = 8.0;
const DRAG_WIDTH: f32 = 112.0;
const ROW_HEIGHT: f32 = theme::CONTROL_HEIGHT;
const SECTION_HEADER_HEIGHT: f32 = 28.0;

pub(super) fn configure_sidebar(ui: &mut Ui) {
    ui.spacing_mut().item_spacing = egui::vec2(8.0, 6.0);
    ui.spacing_mut().button_padding = egui::vec2(10.0, 5.0);
    ui.spacing_mut().interact_size.y = ROW_HEIGHT;
    ui.spacing_mut().slider_width = CONTROL_MAX_WIDTH;
    ui.spacing_mut().combo_width = CONTROL_MAX_WIDTH + VALUE_GAP + VALUE_WIDTH;
    ui.spacing_mut().text_edit_width = CONTROL_MAX_WIDTH + VALUE_GAP + VALUE_WIDTH;
}

pub(super) fn section(
    ui: &mut Ui,
    title: &str,
    default_open: bool,
    add_contents: impl FnOnce(&mut Ui) -> bool,
) -> bool {
    let mut changed = false;
    ui.add_space(10.0);
    draw_section_divider(ui);
    ui.add_space(6.0);

    let id = ui.make_persistent_id(("parameter-section", title));
    let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(
        ui.ctx(),
        id,
        default_open,
    );
    let (header_rect, header_response) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), SECTION_HEADER_HEIGHT),
        Sense::click(),
    );
    if header_response.clicked() {
        state.toggle(ui);
    }
    paint_section_header(ui, header_rect, title, state.is_open());

    if state.is_open() {
        ui.add_space(5.0);
        changed |= add_contents(ui);
        ui.add_space(4.0);
    }
    state.store(ui.ctx());
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
    let metrics = RowMetrics::for_width(ui.available_width());
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.spacing_mut().slider_width = metrics.main_width;
        ui.spacing_mut().combo_width = metrics.span_width;
        ui.spacing_mut().text_edit_width = metrics.span_width;
        ui.spacing_mut().interact_size.y = ROW_HEIGHT;
        label_cell(ui, label);
        ui.add_space(COLUMN_GAP);
        ui.spacing_mut().item_spacing.x = CONTROL_INNER_GAP;
        add_control(ui)
    })
    .inner
}

pub(super) fn text_row(ui: &mut Ui, label: &str, value: &mut String) -> bool {
    row(ui, label, |ui| {
        ui.add_sized(
            [control_span_width(ui), ROW_HEIGHT],
            egui::TextEdit::singleline(value).vertical_align(Align::Center),
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
    let metrics = RowMetrics::for_width(ui.available_width());
    let drag_speed = slider_drag_speed(&range);
    let decimals = slider_decimals(&range);
    let value_range = range.clone();
    row(ui, label, |ui| {
        let mut changed = false;
        changed |= ui
            .add_sized(
                [metrics.main_width, ROW_HEIGHT],
                Slider::new(value, range).show_value(false),
            )
            .changed();
        ui.add_space(VALUE_GAP - CONTROL_INNER_GAP);
        changed |= ui
            .add_sized(
                [VALUE_WIDTH, ROW_HEIGHT],
                DragValue::new(value)
                    .range(value_range)
                    .speed(drag_speed)
                    .max_decimals(decimals),
            )
            .changed();
        changed
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

pub(super) fn control_span_width(ui: &Ui) -> f32 {
    ui.spacing().text_edit_width
}

fn label_cell(ui: &mut Ui, label: &str) {
    ui.allocate_ui_with_layout(
        Vec2::new(LABEL_WIDTH, ROW_HEIGHT),
        Layout::right_to_left(Align::Center),
        |ui| {
            ui.add(
                egui::Label::new(RichText::new(label).size(12.0).color(theme::TEXT_MUTED))
                    .truncate(),
            );
        },
    );
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

fn paint_section_header(ui: &mut Ui, rect: Rect, title: &str, open: bool) {
    let icon_center = Pos2::new(rect.left() + 7.0, rect.center().y);
    let stroke = (1.25, theme::TEXT_MUTED);
    if open {
        ui.painter().line_segment(
            [
                icon_center + Vec2::new(-4.0, -2.0),
                icon_center + Vec2::new(0.0, 2.0),
            ],
            stroke,
        );
        ui.painter().line_segment(
            [
                icon_center + Vec2::new(0.0, 2.0),
                icon_center + Vec2::new(4.0, -2.0),
            ],
            stroke,
        );
    } else {
        ui.painter().line_segment(
            [
                icon_center + Vec2::new(-2.0, -4.0),
                icon_center + Vec2::new(2.0, 0.0),
            ],
            stroke,
        );
        ui.painter().line_segment(
            [
                icon_center + Vec2::new(2.0, 0.0),
                icon_center + Vec2::new(-2.0, 4.0),
            ],
            stroke,
        );
    }
    ui.painter().text(
        Pos2::new(rect.left() + 18.0, rect.center().y),
        Align2::LEFT_CENTER,
        title,
        FontId::proportional(13.0),
        theme::TEXT,
    );
}

#[derive(Clone, Copy)]
struct RowMetrics {
    main_width: f32,
    span_width: f32,
}

impl RowMetrics {
    fn for_width(width: f32) -> Self {
        let control_area = (width - LABEL_WIDTH - COLUMN_GAP).max(CONTROL_MIN_WIDTH);
        let main_width = if control_area > CONTROL_MIN_WIDTH + VALUE_GAP + VALUE_WIDTH {
            (control_area - VALUE_GAP - VALUE_WIDTH).min(CONTROL_MAX_WIDTH)
        } else {
            control_area.min(CONTROL_MAX_WIDTH)
        };
        let span_width = (main_width + VALUE_GAP + VALUE_WIDTH).min(control_area);
        Self {
            main_width,
            span_width,
        }
    }
}

fn slider_drag_speed(range: &RangeInclusive<f32>) -> f64 {
    let span = (*range.end() - *range.start()).abs();
    if span <= 2.0 {
        0.01
    } else if span <= 20.0 {
        0.05
    } else {
        1.0
    }
}

fn slider_decimals(range: &RangeInclusive<f32>) -> usize {
    let span = (*range.end() - *range.start()).abs();
    if span <= 2.0 {
        3
    } else if span <= 20.0 {
        2
    } else {
        0
    }
}
