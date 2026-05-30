use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Vec2};

use crate::{
    app::{LiveApp, image::fit_size},
    export, panels, timeline,
};

impl LiveApp {
    pub(super) fn draw_top_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top-bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("dhool live preview");
                ui.separator();
                if ui.button("Load").clicked() {
                    match export::load_config() {
                        Ok(Some((path, config))) => {
                            self.config = config;
                            self.source_path = Some(path);
                            self.frame_index = 0;
                            self.reset_playback_queue();
                            self.latest_timing = None;
                            self.reject_older_responses();
                            self.queue_render();
                        }
                        Ok(None) => {}
                        Err(message) => {
                            self.status = "load failed".to_string();
                            self.error = Some(message);
                        }
                    }
                }
                if ui.button("Export").clicked() {
                    match export::export_config(&self.config) {
                        Ok(Some(path)) => {
                            self.status = format!("exported {}", path.display());
                            self.error = None;
                        }
                        Ok(None) => {}
                        Err(message) => {
                            self.status = "export failed".to_string();
                            self.error = Some(message);
                        }
                    }
                }
                ui.separator();
                if let Some(path) = &self.source_path {
                    ui.label(path.display().to_string());
                } else {
                    ui.label("bundled default config");
                }
            });
        });
    }

    pub(super) fn draw_sidebar(&mut self, ctx: &egui::Context) -> bool {
        let mut changed = false;
        egui::SidePanel::left("parameter-sidebar")
            .resizable(true)
            .default_width(340.0)
            .width_range(280.0..=460.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    changed |= panels::draw_all(
                        ui,
                        &mut self.config,
                        &mut self.preview_scale,
                        &mut self.preview_field_scale,
                    );
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
                self.config.scene.duration,
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
