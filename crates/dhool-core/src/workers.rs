pub const MAX_AUTO_CUDA_WORKERS: usize = 6;
pub const MAX_MANUAL_CUDA_WORKERS: usize = 16;

pub fn manual_cuda_worker_count(value: &str, frame_count: Option<usize>) -> Option<usize> {
    let parsed = value.parse::<usize>().ok()?;
    Some(cap_workers_to_frame_count(
        parsed.clamp(1, MAX_MANUAL_CUDA_WORKERS),
        frame_count,
    ))
}

pub fn automatic_cuda_worker_count(multiprocessor_count: i32) -> usize {
    let multiprocessors = multiprocessor_count.max(1) as usize;
    ((multiprocessors + 23) / 24).clamp(2, MAX_AUTO_CUDA_WORKERS)
}

pub fn resolution_limited_workers(width: u32, height: u32, worker_count: usize) -> usize {
    let pixels = u64::from(width) * u64::from(height);
    let resolution_cap = if pixels <= 1920 * 1080 {
        worker_count
    } else if pixels <= 2560 * 1440 {
        worker_count.min(4)
    } else if pixels <= 3840 * 2160 {
        worker_count.min(3)
    } else {
        worker_count.min(2)
    };
    resolution_cap.max(1)
}

pub fn cap_workers_to_frame_count(worker_count: usize, frame_count: Option<usize>) -> usize {
    match frame_count {
        Some(frame_count) => worker_count.min(frame_count.max(1)),
        None => worker_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caps_manual_workers() {
        assert_eq!(manual_cuda_worker_count("0", Some(10)), Some(1));
        assert_eq!(manual_cuda_worker_count("99", Some(10)), Some(10));
        assert_eq!(manual_cuda_worker_count("nope", Some(10)), None);
    }

    #[test]
    fn derives_auto_workers_from_multiprocessors() {
        assert_eq!(automatic_cuda_worker_count(1), 2);
        assert_eq!(automatic_cuda_worker_count(48), 2);
        assert_eq!(automatic_cuda_worker_count(144), 6);
        assert_eq!(automatic_cuda_worker_count(240), 6);
    }

    #[test]
    fn limits_workers_by_resolution() {
        assert_eq!(resolution_limited_workers(1920, 1080, 6), 6);
        assert_eq!(resolution_limited_workers(2560, 1440, 6), 4);
        assert_eq!(resolution_limited_workers(3840, 2160, 6), 3);
        assert_eq!(resolution_limited_workers(5000, 3000, 6), 2);
    }
}
