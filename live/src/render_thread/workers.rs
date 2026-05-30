use std::{env, thread};

use dhool_core::{automatic_cuda_worker_count, manual_cuda_worker_count};
use dhool_cuda::{device_info, render_frame};

use crate::render_thread::{
    RenderResponse,
    queue::{WorkQueue, pop_request},
};

pub(super) fn live_worker_count() -> usize {
    if let Ok(value) = env::var("DHOOL_LIVE_GPU_WORKERS") {
        if let Some(parsed) = manual_cuda_worker_count(&value, None) {
            return parsed;
        }
    }

    let info = device_info();
    if !info.available {
        return 1;
    }

    automatic_cuda_worker_count(info.multiprocessor_count)
}

pub(super) fn spawn_worker(
    worker_index: usize,
    work_queue: WorkQueue,
    response_tx: std::sync::mpsc::Sender<RenderResponse>,
    repaint: egui::Context,
) {
    thread::Builder::new()
        .name(format!("dhool-live-render-{worker_index}"))
        .spawn(move || {
            while let Some(request) = pop_request(&work_queue) {
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
        .expect("failed to spawn CUDA preview render worker");
}
