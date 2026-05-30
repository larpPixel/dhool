mod queue;
mod workers;

use std::{
    sync::mpsc::{Receiver, Sender},
    thread,
};

use dhool_config::Config;

use queue::{close_queue, new_queue, push_request};
use workers::{live_worker_count, spawn_worker};

#[derive(Debug)]
pub struct RenderRequest {
    pub request_id: u64,
    pub config: Config,
    pub frame_index: u32,
    pub total_frames: u32,
}

#[derive(Debug)]
pub enum RenderResponse {
    Frame {
        request_id: u64,
        frame: dhool_cuda::RenderedFrame,
    },
    Error {
        request_id: u64,
        message: String,
    },
}

impl RenderResponse {
    pub fn request_id(&self) -> u64 {
        match self {
            RenderResponse::Frame { request_id, .. } | RenderResponse::Error { request_id, .. } => {
                *request_id
            }
        }
    }
}

pub fn spawn_render_thread(
    request_rx: Receiver<RenderRequest>,
    response_tx: Sender<RenderResponse>,
    repaint: egui::Context,
) -> usize {
    let worker_count = live_worker_count();
    let work_queue = new_queue();

    for worker_index in 0..worker_count {
        spawn_worker(
            worker_index,
            work_queue.clone(),
            response_tx.clone(),
            repaint.clone(),
        );
    }

    thread::Builder::new()
        .name("dhool-live-dispatch".to_string())
        .spawn(move || {
            while let Ok(request) = request_rx.recv() {
                push_request(&work_queue, request);
            }
            close_queue(&work_queue);
        })
        .expect("failed to spawn CUDA preview dispatch thread");

    worker_count
}
