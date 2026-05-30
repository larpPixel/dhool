use std::time::{Duration, Instant};

use dhool_core::resolution_limited_workers;
use dhool_cuda::RenderedFrame;
use eframe::egui;

use crate::{
    app::{
        LiveApp,
        image::{color_image_from_opaque_rgba, scaled_dimension},
    },
    render_thread::{RenderRequest, RenderResponse},
    video_render::VideoRenderProgress,
};

impl LiveApp {
    pub(super) fn video_render_active(&self) -> bool {
        self.video_render_rx.is_some()
    }

    pub(super) fn receive_video_render_progress(&mut self, ctx: &egui::Context) {
        let Some(rx) = self.video_render_rx.take() else {
            return;
        };

        let mut keep_receiver = true;
        while let Ok(progress) = rx.try_recv() {
            match progress {
                VideoRenderProgress::Started {
                    frame_count,
                    output_path,
                } => {
                    self.status = format!(
                        "rendering video: 0/{frame_count} -> {}",
                        output_path.display()
                    );
                    self.error = None;
                }
                VideoRenderProgress::Frame {
                    frame_index,
                    frame_count,
                    gpu_ms,
                    readback_ms,
                } => {
                    self.status = format!(
                        "rendering video: {}/{} gpu {:.1} ms readback {:.1} ms",
                        frame_index + 1,
                        frame_count,
                        gpu_ms,
                        readback_ms
                    );
                }
                VideoRenderProgress::Finished {
                    frame_count,
                    wall_ms,
                    outputs,
                } => {
                    let output = outputs
                        .first()
                        .cloned()
                        .unwrap_or_else(|| "video output".to_string());
                    self.status = format!(
                        "rendered {frame_count} frames in {:.1}s: {output}",
                        wall_ms / 1000.0
                    );
                    self.error = None;
                    keep_receiver = false;
                }
                VideoRenderProgress::Error(message) => {
                    self.status = "video render failed".to_string();
                    self.error = Some(message);
                    keep_receiver = false;
                }
            }
        }

        if keep_receiver {
            self.video_render_rx = Some(rx);
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }

    pub(super) fn receive_frames(&mut self, ctx: &egui::Context) -> bool {
        let mut rendered_frame = false;
        let mut completed_request = false;
        let mut newest_frame: Option<(u64, RenderedFrame)> = None;
        let mut newest_error: Option<(u64, String)> = None;

        while let Ok(response) = self.response_rx.try_recv() {
            let response_id = response.request_id();
            self.complete_request(response_id);
            completed_request = true;
            if response_id <= self.displayed_request_id {
                continue;
            }

            match response {
                RenderResponse::Frame { request_id, frame } => {
                    if newest_frame
                        .as_ref()
                        .is_none_or(|(newest_id, _)| request_id > *newest_id)
                    {
                        newest_frame = Some((request_id, frame));
                    }
                }
                RenderResponse::Error {
                    request_id,
                    message,
                } => {
                    if newest_error
                        .as_ref()
                        .is_none_or(|(newest_id, _)| request_id > *newest_id)
                    {
                        newest_error = Some((request_id, message));
                    }
                }
            }
        }

        if completed_request && self.playing && self.max_gpu {
            self.pending_render = true;
        }

        let newest_frame_id = newest_frame.as_ref().map(|(request_id, _)| *request_id);
        let newest_error_id = newest_error.as_ref().map(|(request_id, _)| *request_id);
        if newest_error_id > newest_frame_id {
            if let Some((request_id, message)) = newest_error {
                self.displayed_request_id = request_id;
                self.playing = false;
                self.pending_render = false;
                self.status = "render failed".to_string();
                self.error = Some(message);
            }
            return false;
        }

        if let Some((request_id, frame)) = newest_frame {
            let width = frame.width;
            let height = frame.height;
            let timing = frame.timing;
            let image = color_image_from_opaque_rgba([width as usize, height as usize], frame.rgba);
            if let Some(texture) = self.texture.as_mut() {
                texture.set(image, egui::TextureOptions::LINEAR);
            } else {
                self.texture =
                    Some(ctx.load_texture("dhool-preview", image, egui::TextureOptions::LINEAR));
            }
            self.frame_index = timing.frame_index.min(self.frame_count().saturating_sub(1));
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
        rendered_frame
    }

    pub(super) fn queue_render(&mut self) {
        self.pending_render = true;
        self.try_submit_render();
    }

    pub(super) fn render_current_frame(&mut self) {
        self.playing = false;
        self.reset_playback_queue();
        self.reject_older_responses();
        self.queue_render();
    }

    pub(super) fn try_submit_render(&mut self) {
        while self.pending_render && self.can_submit_render() {
            if !self.max_gpu && self.playing {
                let target_period = Duration::from_secs_f32(1.0 / self.target_preview_fps.max(1.0));
                if self.last_submit_at.elapsed() < target_period {
                    return;
                }
            }

            let frame_count = self.frame_count();
            self.frame_index = self.frame_index.min(frame_count.saturating_sub(1));
            self.next_play_frame_index = self
                .next_play_frame_index
                .min(frame_count.saturating_sub(1));
            let request_frame_index = if self.playing && self.max_gpu {
                self.next_play_frame_index
            } else {
                self.frame_index
            };

            let mut preview_config = self.config.clone();
            preview_config.render.width =
                scaled_dimension(self.config.render.width, self.preview_scale);
            preview_config.render.height =
                scaled_dimension(self.config.render.height, self.preview_scale);
            preview_config.simulation.field_width =
                scaled_dimension(self.config.simulation.field_width, self.preview_field_scale);
            preview_config.simulation.field_height = scaled_dimension(
                self.config.simulation.field_height,
                self.preview_field_scale,
            );

            if let Err(err) = preview_config.validate() {
                self.error = Some(err.to_string());
                self.status = "config is invalid".to_string();
                self.pending_render = false;
                return;
            }

            self.next_request_id += 1;
            self.pending_render = false;
            self.last_submit_at = Instant::now();
            let request_id = self.next_request_id;
            let request = RenderRequest {
                request_id,
                config: preview_config,
                frame_index: request_frame_index,
                total_frames: frame_count,
            };

            match self.render_tx.send(request) {
                Ok(()) => {
                    self.in_flight_request_ids.push(request_id);
                    self.status = format!(
                        "rendering frame {} ({}/{})",
                        request_frame_index + 1,
                        self.in_flight_request_ids.len(),
                        self.active_pipeline_depth()
                    );
                    if self.playing && self.max_gpu {
                        self.advance_next_play_frame(request_frame_index, frame_count);
                        if self.playing && self.can_submit_render() {
                            self.pending_render = true;
                        }
                    }
                }
                Err(err) => {
                    self.status = "render thread stopped".to_string();
                    self.error = Some(err.to_string());
                    return;
                }
            }
        }
    }

    pub(super) fn active_pipeline_depth(&self) -> usize {
        if !self.playing || !self.max_gpu {
            return 1;
        }

        resolution_limited_workers(
            scaled_dimension(self.config.render.width, self.preview_scale),
            scaled_dimension(self.config.render.height, self.preview_scale),
            self.render_worker_count,
        )
    }

    pub(super) fn reject_older_responses(&mut self) {
        self.displayed_request_id = self.next_request_id;
    }

    fn can_submit_render(&self) -> bool {
        self.in_flight_request_ids.len() < self.active_pipeline_depth()
    }

    fn complete_request(&mut self, request_id: u64) {
        if let Some(index) = self
            .in_flight_request_ids
            .iter()
            .position(|in_flight_id| *in_flight_id == request_id)
        {
            self.in_flight_request_ids.swap_remove(index);
        }
    }
}
