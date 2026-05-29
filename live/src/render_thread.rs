use std::{
    sync::mpsc::{Receiver, Sender},
    thread,
};

use dhool_config::Config;
use dhool_cuda::{RenderedFrame, render_frame};

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
        frame: RenderedFrame,
    },
    Error {
        request_id: u64,
        message: String,
    },
}

pub fn spawn_render_thread(
    request_rx: Receiver<RenderRequest>,
    response_tx: Sender<RenderResponse>,
    repaint: egui::Context,
) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name("dhool-live-render".to_string())
        .spawn(move || {
            while let Ok(mut request) = request_rx.recv() {
                while let Ok(next) = request_rx.try_recv() {
                    request = next;
                }

                let response = match render_frame(
                    &request.config,
                    request.frame_index,
                    request.total_frames,
                ) {
                    Ok(frame) => RenderResponse::Frame {
                        request_id: request.request_id,
                        frame,
                    },
                    Err(err) => RenderResponse::Error {
                        request_id: request.request_id,
                        message: err.to_string(),
                    },
                };

                if response_tx.send(response).is_err() {
                    break;
                }
                repaint.request_repaint();
            }
        })
        .expect("failed to spawn CUDA preview render thread")
}
