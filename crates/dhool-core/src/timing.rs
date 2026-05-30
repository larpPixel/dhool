use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FrameTiming {
    pub frame_index: u32,
    pub gpu_ms: f32,
    pub readback_ms: f32,
    pub total_ms: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingSummary {
    pub frames: u32,
    pub total_wall_ms: f32,
    pub avg_gpu_ms: f32,
    pub avg_readback_ms: f32,
    pub avg_total_ms: f32,
    pub max_total_ms: f32,
}

#[derive(Debug)]
pub struct Stopwatch {
    started: Instant,
}

impl Stopwatch {
    pub fn start() -> Self {
        Self {
            started: Instant::now(),
        }
    }

    pub fn elapsed_ms(&self) -> f32 {
        duration_ms(self.started.elapsed())
    }
}

pub fn summarize_timings(frames: &[FrameTiming], wall_ms: f32) -> TimingSummary {
    if frames.is_empty() {
        return TimingSummary {
            frames: 0,
            total_wall_ms: wall_ms,
            avg_gpu_ms: 0.0,
            avg_readback_ms: 0.0,
            avg_total_ms: 0.0,
            max_total_ms: 0.0,
        };
    }
    let len = frames.len() as f32;
    let gpu = frames.iter().map(|t| t.gpu_ms).sum::<f32>() / len;
    let readback = frames.iter().map(|t| t.readback_ms).sum::<f32>() / len;
    let total = frames.iter().map(|t| t.total_ms).sum::<f32>() / len;
    let max_total = frames
        .iter()
        .map(|t| t.total_ms)
        .fold(0.0_f32, |acc, value| acc.max(value));
    TimingSummary {
        frames: frames.len() as u32,
        total_wall_ms: wall_ms,
        avg_gpu_ms: gpu,
        avg_readback_ms: readback,
        avg_total_ms: total,
        max_total_ms: max_total,
    }
}

fn duration_ms(duration: Duration) -> f32 {
    duration.as_secs_f32() * 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarizes_timings() {
        let summary = summarize_timings(
            &[
                FrameTiming {
                    frame_index: 0,
                    gpu_ms: 2.0,
                    readback_ms: 3.0,
                    total_ms: 8.0,
                },
                FrameTiming {
                    frame_index: 1,
                    gpu_ms: 4.0,
                    readback_ms: 5.0,
                    total_ms: 12.0,
                },
            ],
            21.0,
        );
        assert_eq!(summary.frames, 2);
        assert_eq!(summary.avg_gpu_ms, 3.0);
        assert_eq!(summary.max_total_ms, 12.0);
    }
}
