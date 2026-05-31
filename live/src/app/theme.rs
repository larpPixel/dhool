use eframe::egui::{
    self, Color32, CornerRadius, FontFamily, FontId, Margin, RichText, Stroke, TextStyle, Vec2,
};

pub(crate) const BACKGROUND: Color32 = Color32::from_rgb(6, 8, 10);
pub(crate) const PANEL: Color32 = Color32::from_rgb(15, 18, 22);
pub(crate) const PANEL_DEEP: Color32 = Color32::from_rgb(10, 12, 15);
pub(crate) const SECTION: Color32 = Color32::from_rgb(19, 22, 27);
pub(crate) const INPUT: Color32 = Color32::from_rgb(23, 27, 33);
pub(crate) const BORDER: Color32 = Color32::from_rgb(44, 51, 59);
pub(crate) const BORDER_SOFT: Color32 = Color32::from_rgb(32, 38, 45);
pub(crate) const TEXT: Color32 = Color32::from_rgb(230, 235, 238);
pub(crate) const TEXT_MUTED: Color32 = Color32::from_rgb(142, 151, 159);
pub(crate) const ACCENT: Color32 = Color32::from_rgb(236, 126, 58);
pub(crate) const ACCENT_HOVER: Color32 = Color32::from_rgb(255, 146, 74);
pub(crate) const ERROR: Color32 = Color32::from_rgb(255, 128, 100);

pub(crate) const CONTROL_HEIGHT: f32 = 26.0;

#[derive(Clone, Copy)]
pub(crate) enum ButtonKind {
    Primary,
    Secondary,
    Quiet,
}

pub(crate) fn apply(ctx: &egui::Context) {
    ctx.set_theme(egui::Theme::Dark);
    ctx.all_styles_mut(|style| {
        style.text_styles = [
            (
                TextStyle::Heading,
                FontId::new(20.0, FontFamily::Proportional),
            ),
            (TextStyle::Body, FontId::new(13.0, FontFamily::Proportional)),
            (
                TextStyle::Button,
                FontId::new(12.5, FontFamily::Proportional),
            ),
            (
                TextStyle::Small,
                FontId::new(11.0, FontFamily::Proportional),
            ),
            (
                TextStyle::Monospace,
                FontId::new(12.5, FontFamily::Monospace),
            ),
        ]
        .into();

        style.spacing.item_spacing = egui::vec2(8.0, 7.0);
        style.spacing.button_padding = egui::vec2(10.0, 5.0);
        style.spacing.interact_size = egui::vec2(48.0, CONTROL_HEIGHT);
        style.spacing.slider_width = 184.0;
        style.spacing.combo_width = 184.0;
        style.spacing.text_edit_width = 184.0;
        style.spacing.window_margin = Margin::symmetric(14, 12);
        style.spacing.menu_margin = Margin::symmetric(8, 8);
        style.spacing.indent = 10.0;
        style.spacing.scroll = egui::style::ScrollStyle::thin();
        style.animation_time = 0.08;

        let visuals = &mut style.visuals;
        visuals.dark_mode = true;
        visuals.override_text_color = None;
        visuals.panel_fill = BACKGROUND;
        visuals.window_fill = PANEL;
        visuals.window_stroke = Stroke::new(1.0, BORDER);
        visuals.window_corner_radius = CornerRadius::same(6);
        visuals.menu_corner_radius = CornerRadius::same(6);
        visuals.faint_bg_color = PANEL_DEEP;
        visuals.extreme_bg_color = INPUT;
        visuals.code_bg_color = PANEL_DEEP;
        visuals.warn_fg_color = ACCENT_HOVER;
        visuals.error_fg_color = ERROR;
        visuals.hyperlink_color = ACCENT_HOVER;
        visuals.collapsing_header_frame = false;
        visuals.indent_has_left_vline = false;
        visuals.slider_trailing_fill = true;
        visuals.handle_shape = egui::style::HandleShape::Rect { aspect_ratio: 0.45 };

        let widgets = &mut visuals.widgets;
        widgets.noninteractive.weak_bg_fill = PANEL;
        widgets.noninteractive.bg_fill = PANEL;
        widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER_SOFT);
        widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT);
        widgets.noninteractive.corner_radius = CornerRadius::same(4);

        widgets.inactive.weak_bg_fill = INPUT;
        widgets.inactive.bg_fill = INPUT;
        widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);
        widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT);
        widgets.inactive.corner_radius = CornerRadius::same(4);
        widgets.inactive.expansion = 0.0;

        widgets.hovered.weak_bg_fill = Color32::from_rgb(31, 36, 43);
        widgets.hovered.bg_fill = Color32::from_rgb(31, 36, 43);
        widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(74, 83, 94));
        widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::WHITE);
        widgets.hovered.corner_radius = CornerRadius::same(4);
        widgets.hovered.expansion = 0.0;

        widgets.active.weak_bg_fill = Color32::from_rgb(46, 52, 59);
        widgets.active.bg_fill = Color32::from_rgb(46, 52, 59);
        widgets.active.bg_stroke = Stroke::new(1.0, ACCENT);
        widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
        widgets.active.corner_radius = CornerRadius::same(4);
        widgets.active.expansion = 0.0;

        widgets.open.weak_bg_fill = SECTION;
        widgets.open.bg_fill = SECTION;
        widgets.open.bg_stroke = Stroke::new(1.0, BORDER);
        widgets.open.fg_stroke = Stroke::new(1.0, TEXT);
        widgets.open.corner_radius = CornerRadius::same(4);
    });
}

pub(crate) fn app_frame(fill: Color32) -> egui::Frame {
    egui::Frame::new().fill(fill)
}

pub(crate) fn bar_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(PANEL)
        .stroke(Stroke::new(1.0, BORDER_SOFT))
        .inner_margin(Margin::symmetric(14, 8))
}

pub(crate) fn sidebar_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(PANEL_DEEP)
        .stroke(Stroke::new(1.0, BORDER_SOFT))
        .inner_margin(Margin::symmetric(12, 12))
}

pub(crate) fn section_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(SECTION)
        .stroke(Stroke::new(1.0, BORDER_SOFT))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::symmetric(10, 8))
        .outer_margin(Margin::symmetric(0, 4))
}

pub(crate) fn button(
    ui: &mut egui::Ui,
    label: &str,
    size: Vec2,
    enabled: bool,
    kind: ButtonKind,
) -> egui::Response {
    let (fill, stroke, text) = match kind {
        ButtonKind::Primary => (ACCENT, Color32::from_rgb(255, 163, 95), Color32::BLACK),
        ButtonKind::Secondary => (INPUT, BORDER, TEXT),
        ButtonKind::Quiet => (Color32::TRANSPARENT, BORDER_SOFT, TEXT_MUTED),
    };
    let button = egui::Button::new(RichText::new(label).color(text))
        .min_size(size)
        .fill(fill)
        .stroke(Stroke::new(1.0, stroke))
        .corner_radius(CornerRadius::same(4));
    ui.add_enabled(enabled, button)
}
