mod image;
mod layout;
mod playback;
mod rendering;
mod state;

use std::{
    path::PathBuf,
    sync::mpsc::{Receiver, Sender},
    time::Instant,
};

use dhool_config::Config;
use dhool_core::FrameTiming;
use eframe::egui::TextureHandle;

use crate::render_thread::{RenderRequest, RenderResponse};

pub struct LiveApp {
    config: Config,
    source_path: Option<PathBuf>,
    render_tx: Sender<RenderRequest>,
    response_rx: Receiver<RenderResponse>,
    texture: Option<TextureHandle>,
    render_worker_count: usize,
    preview_scale: f32,
    preview_field_scale: f32,
    frame_index: u32,
    next_play_frame_index: u32,
    playing: bool,
    max_gpu: bool,
    target_preview_fps: f32,
    loop_playback: bool,
    last_play_tick: Instant,
    last_submit_at: Instant,
    latest_timing: Option<FrameTiming>,
    next_request_id: u64,
    displayed_request_id: u64,
    in_flight_request_ids: Vec<u64>,
    pending_render: bool,
    status: String,
    error: Option<String>,
}

impl eframe::App for LiveApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.receive_frames(ctx);
        let playback_changed = if self.playing && !self.max_gpu {
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
            self.reset_playback_queue();
            self.reject_older_responses();
            self.queue_render();
        } else if self.playing && self.max_gpu {
            self.pending_render = true;
        }
        self.try_submit_render();

        if self.playing || self.pending_render || !self.in_flight_request_ids.is_empty() {
            let repaint_after = if self.max_gpu {
                std::time::Duration::from_millis(8)
            } else {
                std::time::Duration::from_millis(16)
            };
            ctx.request_repaint_after(repaint_after);
        }
    }
}
