use std::{path::PathBuf, sync::mpsc::channel, time::Instant};

use crate::{
    app::LiveApp,
    render_thread::{RenderRequest, spawn_render_thread},
};
use dhool_config::Config;

impl LiveApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        config: Config,
        source_path: Option<PathBuf>,
    ) -> Self {
        let (render_tx, request_rx) = channel::<RenderRequest>();
        let (response_tx, response_rx) = channel();
        let render_worker_count = spawn_render_thread(request_rx, response_tx, cc.egui_ctx.clone());

        let mut app = Self {
            config,
            source_path,
            render_tx,
            response_rx,
            texture: None,
            render_worker_count,
            preview_scale: 1.0,
            preview_field_scale: 0.5,
            frame_index: 0,
            next_play_frame_index: 0,
            playing: false,
            max_gpu: true,
            target_preview_fps: 60.0,
            loop_playback: true,
            last_play_tick: Instant::now(),
            last_submit_at: Instant::now(),
            latest_timing: None,
            next_request_id: 0,
            displayed_request_id: 0,
            in_flight_request_ids: Vec::new(),
            pending_render: false,
            status: "queued initial render".to_string(),
            error: None,
        };
        app.queue_render();
        app
    }
}
