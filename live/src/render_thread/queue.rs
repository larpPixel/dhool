use std::{
    collections::VecDeque,
    sync::{Arc, Condvar, Mutex},
};

use crate::render_thread::RenderRequest;

pub(super) struct QueueState {
    requests: VecDeque<RenderRequest>,
    closed: bool,
}

pub(super) type WorkQueue = Arc<(Mutex<QueueState>, Condvar)>;

pub(super) fn new_queue() -> WorkQueue {
    Arc::new((
        Mutex::new(QueueState {
            requests: VecDeque::new(),
            closed: false,
        }),
        Condvar::new(),
    ))
}

pub(super) fn push_request(work_queue: &WorkQueue, request: RenderRequest) {
    let (queue, signal) = &**work_queue;
    let mut state = queue.lock().expect("live render queue is poisoned");
    state.requests.push_back(request);
    signal.notify_one();
}

pub(super) fn pop_request(work_queue: &WorkQueue) -> Option<RenderRequest> {
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

pub(super) fn close_queue(work_queue: &WorkQueue) {
    let (queue, signal) = &**work_queue;
    let mut state = queue.lock().expect("live render queue is poisoned");
    state.closed = true;
    signal.notify_all();
}
