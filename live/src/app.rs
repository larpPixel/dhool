use std::{
    path::PathBuf,
    sync::mpsc::{Receiver, Sender, channel},
    time::{Duration, Instant},
};

use dhool_config::Config;
use dhool_core::FrameTiming;
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, TextureHandle, Vec2};

use crate::{
    export, panels,
    render_thread::{RenderRequest, RenderResponse, spawn_render_thread},
    timeline,
};

pub struct LiveApp {
    config: Config,
    source_path: Option<PathBuf>,
    render_tx: Sender<RenderRequest>,
    response_rx: Receiver<RenderResponse>,
    texture: Option<TextureHandle>,
    preview_scale: f32,
    frame_index: u32,
    playing: bool,
    loop_playback: bool,
    last_play_tick: Instant,
    latest_timing: Option<FrameTiming>,
    next_request_id: u64,
    latest_request_id: u64,
    displayed_request_id: u64,
    status: String,
    error: Option<String>,
}

impl LiveApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        config: Config,
        source_path: Option<PathBuf>,
    ) -> Self {
        let (render_tx, request_rx) = channel();
        let (response_tx, response_rx) = channel();
        spawn_render_thread(request_rx, response_tx, cc.egui_ctx.clone());

        let mut app = Self {
            config,
            source_path,
            render_tx,
            response_rx,
            texture: None,
            preview_scale: 0.5,
            frame_index: 0,
            playing: false,
            loop_playback: true,
            last_play_tick: Instant::now(),
            latest_timing: None,
            next_request_id: 0,
            latest_request_id: 0,
            displayed_request_id: 0,
            status: "queued initial render".to_string(),
            error: None,
        };
        app.request_render();
        app
    }

    fn receive_frames(&mut self, ctx: &egui::Context) {
        while let Ok(response) = self.response_rx.try_recv() {
            match response {
                RenderResponse::Frame { request_id, frame } => {
                    if request_id < self.latest_request_id {
                        continue;
                    }
                    let image = egui::ColorImage::from_rgba_unmultiplied(
                        [frame.width as usize, frame.height as usize],
                        &frame.rgba,
                    );
                    if let Some(texture) = self.texture.as_mut() {
                        texture.set(image, egui::TextureOptions::LINEAR);
                    } else {
                        self.texture = Some(ctx.load_texture(
                            "dhool-preview",
                            image,
                            egui::TextureOptions::LINEAR,
                        ));
                    }
                    self.latest_timing = Some(frame.timing);
                    self.displayed_request_id = request_id;
                    self.status = format!(
                        "rendered {}x{} frame {}",
                        frame.width,
                        frame.height,
                        frame.timing.frame_index + 1
                    );
                    self.error = None;
                }
                RenderResponse::Error {
                    request_id,
                    message,
                } => {
                    if request_id < self.latest_request_id {
                        continue;
                    }
                    self.displayed_request_id = request_id;
                    self.status = "render failed".to_string();
                    self.error = Some(message);
                }
            }
        }
    }

    fn request_render(&mut self) {
        let frame_count = self.frame_count();
        self.frame_index = self.frame_index.min(frame_count.saturating_sub(1));

        let mut preview_config = self.config.clone();
        preview_config.render.width =
            scaled_dimension(self.config.render.width, self.preview_scale);
        preview_config.render.height =
            scaled_dimension(self.config.render.height, self.preview_scale);

        if let Err(err) = preview_config.validate() {
            self.error = Some(err.to_string());
            self.status = "config is invalid".to_string();
            return;
        }

        self.next_request_id += 1;
        self.latest_request_id = self.next_request_id;
        let request = RenderRequest {
            request_id: self.next_request_id,
            config: preview_config,
            frame_index: self.frame_index,
            total_frames: frame_count,
        };

        match self.render_tx.send(request) {
            Ok(()) => {
                self.status = format!("rendering frame {}", self.frame_index + 1);
            }
            Err(err) => {
                self.status = "render thread stopped".to_string();
                self.error = Some(err.to_string());
            }
        }
    }

    fn frame_count(&self) -> u32 {
        self.config.frame_count(None).max(1)
    }

    fn advance_playback(&mut self) -> bool {
        if !self.playing {
            self.last_play_tick = Instant::now();
            return false;
        }

        let fps = self.config.scene.fps.max(1.0);
        let frame_period = Duration::from_secs_f32(1.0 / fps);
        let now = Instant::now();
        let elapsed = now.saturating_duration_since(self.last_play_tick);
        if elapsed < frame_period {
            return false;
        }

        let steps = (elapsed.as_secs_f32() / frame_period.as_secs_f32()).floor() as u32;
        self.last_play_tick = now;
        if steps == 0 {
            return false;
        }

        let frame_count = self.frame_count();
        let next = self.frame_index.saturating_add(steps);
        if next >= frame_count {
            if self.loop_playback {
                self.frame_index = next % frame_count;
            } else {
                self.frame_index = frame_count.saturating_sub(1);
                self.playing = false;
            }
        } else {
            self.frame_index = next;
        }
        true
    }

    fn draw_top_bar(&mut self, ctx: &egui::Context) {
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
                            self.latest_timing = None;
                            self.request_render();
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

    fn draw_sidebar(&mut self, ctx: &egui::Context) -> bool {
        let mut changed = false;
        egui::SidePanel::left("parameter-sidebar")
            .resizable(true)
            .default_width(340.0)
            .width_range(280.0..=460.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    changed |= panels::draw_all(ui, &mut self.config, &mut self.preview_scale);
                });
            });
        changed
    }

    fn draw_preview(&mut self, ctx: &egui::Context) {
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

    fn draw_timeline(&mut self, ctx: &egui::Context) -> bool {
        let mut changed = false;
        let frame_count = self.frame_count();
        egui::TopBottomPanel::bottom("timeline").show(ctx, |ui| {
            changed |= timeline::draw(
                ui,
                &mut self.frame_index,
                frame_count,
                self.config.scene.fps,
                self.config.scene.duration,
                &mut self.playing,
                &mut self.loop_playback,
                self.latest_timing.as_ref(),
                self.preview_scale,
            );
        });
        changed
    }
}

impl eframe::App for LiveApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.receive_frames(ctx);
        let playback_changed = self.advance_playback();
        self.draw_top_bar(ctx);
        let config_changed = self.draw_sidebar(ctx);
        self.draw_preview(ctx);
        let timeline_changed = self.draw_timeline(ctx);

        if config_changed {
            self.frame_index = self.frame_index.min(self.frame_count().saturating_sub(1));
        }
        if config_changed || timeline_changed || playback_changed {
            self.request_render();
        }

        if self.playing || self.displayed_request_id < self.latest_request_id {
            ctx.request_repaint_after(Duration::from_millis(16));
        }
    }
}

fn scaled_dimension(base: u32, scale: f32) -> u32 {
    ((base as f32 * scale).round() as u32).clamp(16, 8192)
}

fn fit_size(content: Vec2, bounds: Vec2) -> Vec2 {
    if content.x <= 0.0 || content.y <= 0.0 || bounds.x <= 0.0 || bounds.y <= 0.0 {
        return Vec2::ZERO;
    }
    let scale = (bounds.x / content.x).min(bounds.y / content.y);
    content * scale
}
