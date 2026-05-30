mod device;
mod git;
mod io;
mod metadata;
mod names;
mod rng;
mod timing;
mod workers;

pub use device::DeviceInfo;
pub use git::{GitState, capture_git_state};
pub use io::{copy_text, write_json};
pub use metadata::RenderMetadata;
pub use names::sanitize_stem;
pub use rng::{seeded_unit_f32, splitmix64};
pub use timing::{FrameTiming, Stopwatch, TimingSummary, summarize_timings};
pub use workers::{
    MAX_AUTO_CUDA_WORKERS, MAX_MANUAL_CUDA_WORKERS, automatic_cuda_worker_count,
    cap_workers_to_frame_count, manual_cuda_worker_count, resolution_limited_workers,
};
