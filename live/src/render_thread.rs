use std::{
    collections::VecDeque,
    env,
    sync::{
        Arc, Condvar, Mutex,
        mpsc::{Receiver, Sender},
    },
    thread,
};

use dhool_config::Config;
use dhool_cuda::{RenderedFrame, device_info, render_frame};

const MAX_AUTO_WORKERS: usize = 6;
const MAX_MANUAL_WORKERS: usize = 16;

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

impl RenderResponse {
    pub fn request_id(&self) -> u64 {
        match self {
            RenderResponse::Frame { request_id, .. } | RenderResponse::Error { request_id, .. } => {
                *request_id
            }
        }
    }
}

struct QueueState {
    requests: VecDeque<RenderRequest>,
    closed: bool,
}

type WorkQueue = Arc<(Mutex<QueueState>, Condvar)>;

pub fn spawn_render_thread(
    request_rx: Receiver<RenderRequest>,
    response_tx: Sender<RenderResponse>,
    repaint: egui::Context,
) -> usize {
    let worker_count = live_worker_count();
    let work_queue = Arc::new((
        Mutex::new(QueueState {
            requests: VecDeque::new(),
            closed: false,
        }),
        Condvar::new(),
    ));

    for worker_index in 0..worker_count {
        spawn_worker(
            worker_index,
            Arc::clone(&work_queue),
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

fn live_worker_count() -> usize {
    if let Ok(value) = env::var("DHOOL_LIVE_GPU_WORKERS") {
        if let Ok(parsed) = value.parse::<usize>() {
            return parsed.clamp(1, MAX_MANUAL_WORKERS);
        }
    }

    let info = device_info();
    if !info.available {
        return 1;
    }

    let multiprocessors = info.multiprocessor_count.max(1) as usize;
    ((multiprocessors + 23) / 24).clamp(2, MAX_AUTO_WORKERS)
}

fn spawn_worker(
    worker_index: usize,
    work_queue: WorkQueue,
    response_tx: Sender<RenderResponse>,
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

fn push_request(work_queue: &WorkQueue, request: RenderRequest) {
    let (queue, signal) = &**work_queue;
    let mut state = queue.lock().expect("live render queue is poisoned");
    state.requests.push_back(request);
    signal.notify_one();
}

fn pop_request(work_queue: &WorkQueue) -> Option<RenderRequest> {
    let (queue, signal) = &**work_queue;
    let mut state = queue.lock().expect("live render queue is poisoned");
    loop {
        if let Some(request) = state.requests.pop_front() {
            return Some(request);
        }
        if state.closed {
            return None;
        }
        state = signal
            .wait(state)
            .expect("live render queue is poisoned while waiting");
    }
}

fn close_queue(work_queue: &WorkQueue) {
    let (queue, signal) = &**work_queue;
    let mut state = queue.lock().expect("live render queue is poisoned");
    state.closed = true;
    signal.notify_all();
}
