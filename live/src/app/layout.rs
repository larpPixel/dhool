use std::path::PathBuf;

use dhool_config::{BackendKind, ExportMode};
use dhool_core::sanitize_stem;
use eframe::egui::{
    self, Align, Align2, Color32, FontId, Layout, Pos2, Rect, RichText, Sense, Vec2,
};

use crate::{
    app::{LiveApp, image::fit_size, theme},
    export, panels, randomize, timeline, video_render,
};

const TOOLBAR_BUTTON: Vec2 = Vec2::new(94.0, 28.0);
const HOME_BUTTON: Vec2 = Vec2::new(210.0, 34.0);
const MODAL_WIDTH: f32 = 440.0;
const SIDEBAR_WIDTH: f32 = 388.0;
const SIDEBAR_MIN_WIDTH: f32 = 344.0;
const SIDEBAR_MAX_WIDTH: f32 = 520.0;

impl LiveApp {
    pub(super) fn draw_home(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(theme::app_frame(theme::BACKGROUND))
            .show(ctx, |ui| {
                let rect = ui.max_rect();
                paint_home_background(ui, rect);
                ui.with_layout(
                    egui::Layout::centered_and_justified(egui::Direction::TopDown),
                    |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new("dhool")
                                    .size(40.0)
                                    .strong()
                                    .color(theme::TEXT),
                            );
                            ui.label(
                                RichText::new("deterministic CUDA inferno renderer")
                                    .size(13.0)
                                    .color(theme::TEXT_MUTED),
                            );
                            ui.add_space(24.0);
                            if home_button(ui, "Open preview", true, theme::ButtonKind::Primary)
                                .clicked()
                            {
                                self.show_home = false;
                                self.render_current_frame();
                            }
                            if home_button(
                                ui,
                                "Render video",
                                !self.video_render_active(),
                                theme::ButtonKind::Secondary,
                            )
                            .clicked()
                            {
                                self.open_render_settings();
                            }
                            if home_button(
                                ui,
                                "Randomize",
                                !self.video_render_active(),
                                theme::ButtonKind::Secondary,
                            )
                            .clicked()
                            {
                                self.randomize_current_config();
                            }
                            if home_button(ui, "Load config", true, theme::ButtonKind::Secondary)
                                .clicked()
                            {
                                self.load_config_from_dialog();
                            }
                            ui.add_space(16.0);
                            if let Some(error) = &self.error {
                                ui.colored_label(theme::ERROR, error);
                            } else {
                                ui.label(RichText::new(&self.status).color(theme::TEXT_MUTED));
                            }
                        });
                    },
                );
            });
    }

    pub(super) fn draw_top_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top-bar")
            .exact_height(48.0)
            .show_separator_line(false)
            .frame(theme::bar_frame())
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(8.0, 0.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("dhool")
                            .size(16.0)
                            .strong()
                            .color(theme::TEXT),
                    );
                    ui.label(
                        RichText::new("live preview")
                            .size(12.0)
                            .color(theme::TEXT_MUTED),
                    );
                    ui.add_space(10.0);
                    if toolbar_button(ui, "Home", true, theme::ButtonKind::Quiet).clicked() {
                        self.playing = false;
                        self.pending_render = false;
                        self.show_home = true;
                    }
                    if toolbar_button(
                        ui,
                        "Render",
                        !self.video_render_active(),
                        theme::ButtonKind::Primary,
                    )
                    .clicked()
                    {
                        self.open_render_settings();
                    }
                    if toolbar_button(
                        ui,
                        "Randomize",
                        !self.video_render_active(),
                        theme::ButtonKind::Secondary,
                    )
                    .clicked()
                    {
                        self.randomize_current_config();
                    }
                    if toolbar_button(ui, "Load config", true, theme::ButtonKind::Secondary)
                        .clicked()
                    {
                        self.load_config_from_dialog();
                    }
                    if toolbar_button(ui, "Save config", true, theme::ButtonKind::Secondary)
                        .clicked()
                    {
                        match export::export_config(&self.config) {
                            Ok(Some(path)) => {
                                self.status = format!("saved {}", path.display());
                                self.source_path = Some(path);
                                self.error = None;
                            }
                            Ok(None) => {}
                            Err(message) => {
                                self.status = "save failed".to_string();
                                self.error = Some(message);
                            }
                        }
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let source = self
                            .source_path
                            .as_ref()
                            .map(|path| path.display().to_string())
                            .unwrap_or_else(|| "unsaved config".to_string());
                        ui.add_sized(
                            [ui.available_width().min(360.0), theme::CONTROL_HEIGHT],
                            egui::Label::new(RichText::new(source).color(theme::TEXT_MUTED))
                                .truncate(),
                        );
                    });
                });
            });
    }

    fn open_render_settings(&mut self) {
        let mut config = self.config.clone();
        config.export.mode = ExportMode::Pipe;
        self.render_config_draft = Some(config);
        self.render_settings_open = true;
    }

    fn randomize_current_config(&mut self) {
        match randomize::randomize_config(&mut self.config) {
            Ok(()) => {
                self.show_home = false;
                self.source_path = None;
                self.texture = None;
                self.frame_index = 0;
                self.latest_timing = None;
                self.status = "randomized config".to_string();
                self.error = None;
                self.render_current_frame();
            }
            Err(message) => {
                self.status = "randomize failed".to_string();
                self.error = Some(message);
            }
        }
    }

    fn load_config_from_dialog(&mut self) {
        match export::load_config() {
            Ok(Some((path, config))) => {
                self.config = config;
                self.source_path = Some(path);
                self.texture = None;
                self.frame_index = 0;
                self.latest_timing = None;
                self.show_home = false;
                self.render_current_frame();
            }
            Ok(None) => {}
            Err(message) => {
                self.status = "load failed".to_string();
                self.error = Some(message);
            }
        }
    }

    pub(super) fn draw_render_settings(&mut self, ctx: &egui::Context) {
        let mut open = self.render_settings_open;
        let mut render_requested = false;
        let mut close_requested = false;

        if open {
            if self.render_config_draft.is_none() {
                self.open_render_settings();
            }
            let response =
                egui::Modal::new(egui::Id::new("render-settings-modal")).show(ctx, |ui| {
                    ui.set_width(MODAL_WIDTH);
                    ui.label(
                        RichText::new("Render video")
                            .size(20.0)
                            .strong()
                            .color(theme::TEXT),
                    );
                    ui.add_space(8.0);
                    ui.separator();
                    if let Some(config) = self.render_config_draft.as_mut() {
                        panels::draw_render_settings(ui, config);
                    }
                    ui.separator();
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if toolbar_button(ui, "Close", true, theme::ButtonKind::Quiet).clicked() {
                            close_requested = true;
                        }
                        if toolbar_button(
                            ui,
                            "Render video",
                            !self.video_render_active(),
                            theme::ButtonKind::Primary,
                        )
                        .clicked()
                        {
                            render_requested = true;
                        }
                    });
                });
            if response.should_close() {
                close_requested = true;
            }
        }

        if render_requested {
            close_requested = self.start_video_render_from_draft(ctx);
        }

        if close_requested {
            open = false;
        }
        self.render_settings_open = open;
        if !open {
            self.render_config_draft = None;
        }
    }

    fn start_video_render_from_draft(&mut self, ctx: &egui::Context) -> bool {
        if self.video_render_active() {
            return false;
        }

        let Some(mut config) = self.render_config_draft.clone() else {
            return false;
        };
        config.export.mode = ExportMode::Pipe;

        if config.backend.preferred != BackendKind::Cuda {
            self.status = "render failed".to_string();
            self.error = Some("video rendering requires the CUDA backend".to_string());
            return false;
        }

        let default_name = format!("{}.mp4", sanitize_stem(&config.scene.name));
        let Some(mut path) = rfd::FileDialog::new()
            .add_filter("MP4 video", &["mp4"])
            .set_file_name(&default_name)
            .save_file()
        else {
            return false;
        };
        if path.extension().is_none() {
            path.set_extension("mp4");
        }
        let output_dir = output_dir_for_video_path(&path);
        config.export.output_dir = output_dir.to_string_lossy().into_owned();

        if let Err(err) = config.validate() {
            self.status = "render failed".to_string();
            self.error = Some(err.to_string());
            return false;
        }

        self.config = config.clone();
        self.playing = false;
        self.pending_render = false;
        self.reject_older_responses();
        self.video_render_rx = Some(video_render::spawn_video_render(config, path, ctx.clone()));
        self.status = "starting video render".to_string();
        self.error = None;
        true
    }

    pub(super) fn draw_sidebar(&mut self, ctx: &egui::Context) -> bool {
        let mut changed = false;
        egui::SidePanel::left("parameter-sidebar")
            .resizable(true)
            .default_width(SIDEBAR_WIDTH)
            .width_range(SIDEBAR_MIN_WIDTH..=SIDEBAR_MAX_WIDTH)
            .show_separator_line(false)
            .frame(theme::sidebar_frame())
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Parameters")
                            .size(15.0)
                            .strong()
                            .color(theme::TEXT),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!("{} workers", self.render_worker_count))
                                .size(11.0)
                                .color(theme::TEXT_MUTED),
                        );
                    });
                });
                ui.add_space(8.0);
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.add_enabled_ui(!self.video_render_active(), |ui| {
                            changed |= panels::draw_all(
                                ui,
                                &mut self.config,
                                &mut self.preview_scale,
                                &mut self.preview_field_scale,
                            );
                        });
                    });
            });
        changed
    }

    pub(super) fn draw_preview(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(theme::app_frame(theme::BACKGROUND))
            .show(ctx, |ui| {
                let available = ui.available_size();
                let desired = Vec2::new(available.x.max(1.0), available.y.max(1.0));
                let (rect, _) = ui.allocate_exact_size(desired, Sense::hover());
                ui.painter().rect_filled(rect, 0.0, theme::BACKGROUND);

                if let Some(texture) = &self.texture {
                    let image_size = texture.size_vec2();
                    let draw_size = fit_size(image_size, rect.size());
                    let image_rect = Rect::from_center_size(rect.center(), draw_size);
                    ui.painter().image(
                        texture.id(),
                        image_rect,
                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        Color32::WHITE,
                    );
                } else {
                    ui.painter().text(
                        rect.center(),
                        Align2::CENTER_CENTER,
                        &self.status,
                        FontId::proportional(14.0),
                        theme::TEXT_MUTED,
                    );
                }

                if let Some(error) = &self.error {
                    paint_preview_status(ui, rect, error, theme::ERROR);
                } else {
                    paint_preview_status(ui, rect, &self.status, theme::TEXT_MUTED);
                }
            });
    }

    pub(super) fn draw_timeline(&mut self, ctx: &egui::Context) -> bool {
        let mut changed = false;
        let frame_count = self.frame_count();
        let in_flight = self.in_flight_request_ids.len();
        let pipeline_depth = self.active_pipeline_depth();
        egui::TopBottomPanel::bottom("timeline")
            .exact_height(50.0)
            .show_separator_line(false)
            .frame(theme::bar_frame())
            .show(ctx, |ui| {
                changed |= timeline::draw(
                    ui,
                    &mut self.frame_index,
                    frame_count,
                    self.config.scene.fps,
                    &mut self.playing,
                    &mut self.loop_playback,
                    &mut self.max_gpu,
                    &mut self.target_preview_fps,
                    self.latest_timing.as_ref(),
                    self.preview_scale,
                    in_flight,
                    pipeline_depth,
                );
            });
        changed
    }
}

fn output_dir_for_video_path(path: &PathBuf) -> PathBuf {
    path.parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn toolbar_button(
    ui: &mut egui::Ui,
    label: &str,
    enabled: bool,
    kind: theme::ButtonKind,
) -> egui::Response {
    theme::button(ui, label, TOOLBAR_BUTTON, enabled, kind)
}

fn home_button(
    ui: &mut egui::Ui,
    label: &str,
    enabled: bool,
    kind: theme::ButtonKind,
) -> egui::Response {
    theme::button(ui, label, HOME_BUTTON, enabled, kind)
}

fn paint_home_background(ui: &mut egui::Ui, rect: Rect) {
    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, theme::BACKGROUND);
    painter.rect_filled(
        Rect::from_min_size(rect.left_top(), Vec2::new(4.0, rect.height())),
        0.0,
        theme::ACCENT,
    );

    let band_top = rect.bottom() - rect.height() * 0.34;
    for index in 0..9 {
        let t = index as f32 / 8.0;
        let y = band_top + t * rect.height() * 0.24;
        let alpha = (44.0 * (1.0 - t)).round() as u8;
        painter.line_segment(
            [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y - 42.0)],
            (1.0, Color32::from_rgba_premultiplied(236, 126, 58, alpha)),
        );
    }
}

fn paint_preview_status(ui: &mut egui::Ui, rect: Rect, text: &str, color: Color32) {
    let max_width = (rect.width() - 32.0).max(80.0).min(520.0);
    let status_rect = Rect::from_min_size(
        rect.left_top() + Vec2::new(16.0, 16.0),
        Vec2::new(max_width, theme::CONTROL_HEIGHT),
    );
    ui.put(
        status_rect,
        egui::Label::new(RichText::new(text).size(12.0).color(color)).truncate(),
    );
}
