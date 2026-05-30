use std::path::PathBuf;

use dhool_config::{BackendKind, ExportMode};
use dhool_core::sanitize_stem;
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Vec2};

use crate::{
    app::{LiveApp, image::fit_size},
    export, panels, randomize, timeline, video_render,
};

impl LiveApp {
    pub(super) fn draw_home(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ui.max_rect();
            ui.painter()
                .rect_filled(rect, 0.0, Color32::from_rgb(7, 8, 10));
            ui.with_layout(
                egui::Layout::centered_and_justified(egui::Direction::TopDown),
                |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("dhool");
                        ui.add_space(16.0);
                        if ui.button("Open preview").clicked() {
                            self.show_home = false;
                            self.render_current_frame();
                        }
                        if ui
                            .add_enabled(
                                !self.video_render_active(),
                                egui::Button::new("Render video"),
                            )
                            .clicked()
                        {
                            self.open_render_settings();
                        }
                        if ui
                            .add_enabled(
                                !self.video_render_active(),
                                egui::Button::new("Randomize"),
                            )
                            .clicked()
                        {
                            self.randomize_current_config();
                        }
                        if ui.button("Load config").clicked() {
                            self.load_config_from_dialog();
                        }
                        ui.add_space(12.0);
                        if let Some(error) = &self.error {
                            ui.colored_label(Color32::from_rgb(255, 140, 110), error);
                        } else {
                            ui.label(&self.status);
                        }
                    });
                },
            );
        });
    }

    pub(super) fn draw_top_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top-bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("dhool live preview");
                ui.separator();
                if ui.button("Home").clicked() {
                    self.playing = false;
                    self.pending_render = false;
                    self.show_home = true;
                }
                if ui
                    .add_enabled(!self.video_render_active(), egui::Button::new("Render"))
                    .clicked()
                {
                    self.open_render_settings();
                }
                if ui
                    .add_enabled(!self.video_render_active(), egui::Button::new("Randomize"))
                    .clicked()
                {
                    self.randomize_current_config();
                }
                if ui.button("Load config").clicked() {
                    self.load_config_from_dialog();
                }
                if ui.button("Save config").clicked() {
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
                ui.separator();
                if let Some(path) = &self.source_path {
                    ui.label(path.display().to_string());
                } else {
                    ui.label("unsaved config");
                }
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
                    ui.set_width(360.0);
                    ui.heading("Render video");
                    ui.separator();
                    ui.spacing_mut().slider_width = 180.0;
                    if let Some(config) = self.render_config_draft.as_mut() {
                        panels::draw_render_settings(ui, config);
                    }
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui
                            .add_enabled(
                                !self.video_render_active(),
                                egui::Button::new("Render video"),
                            )
                            .clicked()
                        {
                            render_requested = true;
                        }
                        if ui.button("Close").clicked() {
                            close_requested = true;
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
            .default_width(340.0)
            .width_range(280.0..=460.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
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
        egui::CentralPanel::default().show(ctx, |ui| {
            let available = ui.available_size();
            let desired = Vec2::new(available.x.max(1.0), available.y.max(1.0));
            let (rect, _) = ui.allocate_exact_size(desired, Sense::hover());
            ui.painter()
                .rect_filled(rect, 0.0, Color32::from_rgb(7, 8, 10));

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
                    Color32::from_gray(180),
                );
            }

            if let Some(error) = &self.error {
                ui.painter().text(
                    rect.left_top() + Vec2::new(16.0, 16.0),
                    Align2::LEFT_TOP,
                    error,
                    FontId::proportional(14.0),
                    Color32::from_rgb(255, 140, 110),
                );
            } else {
                ui.painter().text(
                    rect.left_top() + Vec2::new(16.0, 16.0),
                    Align2::LEFT_TOP,
                    &self.status,
                    FontId::proportional(13.0),
                    Color32::from_gray(190),
                );
            }
        });
    }

    pub(super) fn draw_timeline(&mut self, ctx: &egui::Context) -> bool {
        let mut changed = false;
        let frame_count = self.frame_count();
        let in_flight = self.in_flight_request_ids.len();
        let pipeline_depth = self.active_pipeline_depth();
        egui::TopBottomPanel::bottom("timeline").show(ctx, |ui| {
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
