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
    max_gpu: bool,
    target_preview_fps: f32,
    loop_playback: bool,
    last_play_tick: Instant,
    last_submit_at: Instant,
    latest_timing: Option<FrameTiming>,
    next_request_id: u64,
    displayed_request_id: u64,
    in_flight_request_id: Option<u64>,
    pending_render: bool,
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
            max_gpu: true,
            target_preview_fps: 60.0,
            loop_playback: true,
            last_play_tick: Instant::now(),
            last_submit_at: Instant::now(),
            latest_timing: None,
            next_request_id: 0,
            displayed_request_id: 0,
            in_flight_request_id: None,
            pending_render: false,
            status: "queued initial render".to_string(),
            error: None,
        };
        app.queue_render();
        app
    }

    fn receive_frames(&mut self, ctx: &egui::Context) -> bool {
        let mut rendered_frame = false;
        while let Ok(response) = self.response_rx.try_recv() {
            match response {
                RenderResponse::Frame { request_id, frame } => {
                    if Some(request_id) == self.in_flight_request_id {
                        self.in_flight_request_id = None;
                    }
                    if request_id < self.displayed_request_id {
                        continue;
                    }
                    let width = frame.width;
                    let height = frame.height;
                    let timing = frame.timing;
                    let image = egui::ColorImage::from_rgba_unmultiplied(
                        [width as usize, height as usize],
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
                    self.latest_timing = Some(timing);
                    self.displayed_request_id = request_id;
                    rendered_frame = true;
                    self.status = format!(
                        "rendered {}x{} frame {}",
                        width,
                        height,
                        timing.frame_index + 1
                    );
                    self.error = None;
                }
                RenderResponse::Error {
                    request_id,
                    message,
                } => {
                    if Some(request_id) == self.in_flight_request_id {
                        self.in_flight_request_id = None;
                    }
                    if request_id < self.displayed_request_id {
                        continue;
                    }
                    self.displayed_request_id = request_id;
                    self.playing = false;
                    self.status = "render failed".to_string();
                    self.error = Some(message);
                }
            }
        }
        rendered_frame
    }

    fn queue_render(&mut self) {
        self.pending_render = true;
        self.try_submit_render();
    }

    fn try_submit_render(&mut self) {
        if !self.pending_render || self.in_flight_request_id.is_some() {
            return;
        }
        if !self.max_gpu && self.playing {
            let target_period = Duration::from_secs_f32(1.0 / self.target_preview_fps.max(1.0));
            if self.last_submit_at.elapsed() < target_period {
                return;
            }
        }

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
            self.pending_render = false;
            return;
        }

        self.next_request_id += 1;
        self.pending_render = false;
        self.last_submit_at = Instant::now();
        let request = RenderRequest {
            request_id: self.next_request_id,
            config: preview_config,
            frame_index: self.frame_index,
            total_frames: frame_count,
        };

        match self.render_tx.send(request) {
            Ok(()) => {
                self.in_flight_request_id = Some(self.next_request_id);
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

    fn advance_one_frame(&mut self) -> bool {
        self.advance_frames(1)
    }

    fn advance_playback_realtime(&mut self) -> bool {
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

        self.advance_frames(steps)
    }

    fn advance_frames(&mut self, steps: u32) -> bool {
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
                &mut self.max_gpu,
                &mut self.target_preview_fps,
                self.latest_timing.as_ref(),
                self.preview_scale,
            );
        });
        changed
    }
}

impl eframe::App for LiveApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let rendered_frame = self.receive_frames(ctx);
        let playback_changed = if self.playing && self.max_gpu && rendered_frame {
            self.advance_one_frame()
        } else if self.playing && !self.max_gpu {
            self.advance_playback_realtime()
        } else {
            false
        };
        if playback_changed {
            self.queue_render();
        }
        self.draw_top_bar(ctx);
        let config_changed = self.draw_sidebar(ctx);
        self.draw_preview(ctx);
        let timeline_changed = self.draw_timeline(ctx);

        if config_changed {
            self.frame_index = self.frame_index.min(self.frame_count().saturating_sub(1));
        }
        if config_changed || timeline_changed {
            self.queue_render();
        } else if self.playing && self.max_gpu && self.in_flight_request_id.is_none() {
            self.queue_render();
        }
        self.try_submit_render();

        if self.playing || self.pending_render || self.in_flight_request_id.is_some() {
            let repaint_after = if self.max_gpu {
                Duration::from_millis(1)
            } else {
                Duration::from_millis(16)
            };
            ctx.request_repaint_after(repaint_after);
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
