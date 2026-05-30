use std::{mem::MaybeUninit, time::Instant};

use anyhow::Result;
use dhool_config::{BackendKind, Config};
use dhool_core::FrameTiming;

use crate::{
    error::CudaBackendError,
    ffi::{RawRenderReport, c_string, dhool_cuda_render_rgba8},
    params::params_from_config,
    types::RenderedFrame,
};

pub fn render_frame(config: &Config, frame_index: u32, total_frames: u32) -> Result<RenderedFrame> {
    if config.backend.preferred == BackendKind::CpuReference {
        return Err(CudaBackendError::CpuReferenceRequested.into());
    }

    let width = config.render.width;
    let height = config.render.height;
    let buffer_len = width as usize * height as usize * 4;
    let mut rgba = vec![0_u8; buffer_len];
    let params = params_from_config(config, frame_index, total_frames)?;

    let mut report = MaybeUninit::<RawRenderReport>::zeroed();
    let wall = Instant::now();
    let status =
        unsafe { dhool_cuda_render_rgba8(&params, rgba.as_mut_ptr(), report.as_mut_ptr()) };
    let wall_ms = wall.elapsed().as_secs_f32() * 1000.0;
    let report = unsafe { report.assume_init() };
    if status == 0 || report.status == 0 {
        let message = c_string(&report.error_message);
        return Err(CudaBackendError::Cuda(if message.is_empty() {
            format!("CUDA render failed with code {}", report.error_code)
        } else {
            message
        })
        .into());
    }

    Ok(RenderedFrame {
        width,
        height,
        rgba,
        timing: FrameTiming {
            frame_index,
            gpu_ms: report.gpu_ms,
            readback_ms: report.readback_ms,
            total_ms: wall_ms,
        },
    })
}

pub fn known_limitations() -> Vec<String> {
    vec![
        "EXR sequence export is rejected until the HDR image sequence writer is added.".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, thread};

    use dhool_config::Quality;

    use super::*;

    #[test]
    #[ignore = "requires a CUDA device"]
    fn renders_parallel_cuda_smoke() {
        let config = small_cuda_config();
        let workers = (0..2)
            .map(|frame_index| {
                let config = config.clone();
                thread::spawn(move || render_frame(&config, frame_index, 2))
            })
            .collect::<Vec<_>>();

        for worker in workers {
            let frame = worker
                .join()
                .expect("render worker panicked")
                .expect("CUDA render failed");
            assert_eq!(frame.width, 96);
            assert_eq!(frame.height, 54);
            assert_eq!(frame.rgba.len(), 96 * 54 * 4);
            assert!(frame.timing.total_ms > 0.0);
        }
    }

    #[test]
    #[ignore = "requires a CUDA device and renders full 1080p frames"]
    fn benchmarks_1080p_cuda_render() {
        let path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../configs/inferno.production.yaml");
        let mut config = Config::load(path).expect("sample config loads");
        config.render.width = 1920;
        config.render.height = 1080;
        config.validate().expect("1080p CUDA config validates");

        let mut gpu_ms = 0.0;
        let mut readback_ms = 0.0;
        let mut total_ms = 0.0;
        let frames = 24;
        let warmup_frames = 12;
        for frame_index in 0..frames {
            let frame = render_frame(&config, frame_index, frames).expect("CUDA render failed");
            println!(
                "frame {frame_index}: gpu {:.3} ms readback {:.3} ms total {:.3} ms",
                frame.timing.gpu_ms, frame.timing.readback_ms, frame.timing.total_ms
            );
            if frame_index >= warmup_frames {
                gpu_ms += frame.timing.gpu_ms;
                readback_ms += frame.timing.readback_ms;
                total_ms += frame.timing.total_ms;
            }
        }

        let measured = (frames - warmup_frames) as f32;
        println!(
            "avg after warmup: gpu {:.3} ms readback {:.3} ms total {:.3} ms fps {:.1}",
            gpu_ms / measured,
            readback_ms / measured,
            total_ms / measured,
            1000.0 / (total_ms / measured)
        );
    }

    fn small_cuda_config() -> Config {
        let path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../configs/inferno.production.yaml");
        let mut config = Config::load(path).expect("sample config loads");
        config.render.width = 96;
        config.render.height = 54;
        config.render.quality = Quality::Preview;
        config.simulation.field_width = 32;
        config.simulation.field_height = 24;
        config.simulation.substeps = 1;
        config.particles.count = 256;
        config.bloom.pyramid_levels = 1;
        config.validate().expect("small CUDA config validates");
        config
    }
}
