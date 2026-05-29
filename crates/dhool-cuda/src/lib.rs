use std::{
    ffi::CStr,
    mem::MaybeUninit,
    os::raw::{c_char, c_int},
    time::Instant,
};

use anyhow::{Context, Result};
use dhool_config::{BackendKind, Config, Quality};
use dhool_core::{DeviceInfo, FrameTiming};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CudaBackendError {
    #[error("CUDA backend is required; cpu-reference is not a production render backend")]
    CpuReferenceRequested,
    #[error("CUDA failed: {0}")]
    Cuda(String),
}

#[derive(Debug)]
pub struct RenderedFrame {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub timing: FrameTiming,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RawDeviceInfo {
    available: c_int,
    name: [c_char; 256],
    driver_version: c_int,
    runtime_version: c_int,
    global_memory_bytes: u64,
    multiprocessor_count: c_int,
    error_code: c_int,
    error_message: [c_char; 512],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RawRenderReport {
    status: c_int,
    gpu_ms: f32,
    readback_ms: f32,
    error_code: c_int,
    error_message: [c_char; 512],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RawRenderParams {
    width: u32,
    height: u32,
    frame_index: u32,
    total_frames: u32,
    seed: u64,
    time_seconds: f32,
    duration_seconds: f32,
    fps: f32,
    quality: u32,
    field_width: u32,
    field_height: u32,
    substeps: u32,
    velocity_decay: f32,
    vorticity: f32,
    buoyancy: f32,
    turbulence: f32,
    swirl: f32,
    advection_dissipation: f32,
    plasma_emission_strength: f32,
    plasma_filament_density: f32,
    plasma_core_temperature: f32,
    plasma_edge_wrapping: f32,
    plasma_color_core: [f32; 3],
    plasma_color_edge: [f32; 3],
    particles_count: u32,
    particles_size_min: f32,
    particles_size_max: f32,
    particles_lifetime: f32,
    particles_flow_coupling: f32,
    particles_ember_brightness: f32,
    particles_gravity: f32,
    particles_drag: f32,
    particles_thermal_lift: f32,
    particles_jitter: f32,
    smoke_density: f32,
    smoke_scattering: f32,
    smoke_absorption: f32,
    smoke_color_cool: [f32; 3],
    smoke_color_warm: [f32; 3],
    bloom_threshold: f32,
    bloom_strength: f32,
    bloom_radius: f32,
    bloom_pyramid_levels: u32,
    grade_exposure: f32,
    grade_contrast: f32,
    grade_saturation: f32,
    grade_black_point: f32,
    grade_vignette: f32,
    grade_grain: f32,
}

unsafe extern "C" {
    fn dhool_cuda_device_info(out: *mut RawDeviceInfo) -> c_int;
    fn dhool_cuda_render_rgba8(
        params: *const RawRenderParams,
        host_rgba: *mut u8,
        report: *mut RawRenderReport,
    ) -> c_int;
}

pub fn device_info() -> DeviceInfo {
    let mut raw = MaybeUninit::<RawDeviceInfo>::zeroed();
    let status = unsafe { dhool_cuda_device_info(raw.as_mut_ptr()) };
    let raw = unsafe { raw.assume_init() };
    let error = c_string(&raw.error_message);
    DeviceInfo {
        backend: "cuda".to_string(),
        available: status != 0 && raw.available != 0,
        name: c_string(&raw.name),
        driver_version: cuda_version(raw.driver_version),
        runtime_version: cuda_version(raw.runtime_version),
        global_memory_bytes: raw.global_memory_bytes,
        multiprocessor_count: raw.multiprocessor_count,
        error: (!error.is_empty()).then_some(error),
    }
}

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

fn params_from_config(
    config: &Config,
    frame_index: u32,
    total_frames: u32,
) -> Result<RawRenderParams> {
    let cool = config
        .smoke_cool_rgb()
        .context("invalid smoke.color_cool")?;
    let warm = config
        .smoke_warm_rgb()
        .context("invalid smoke.color_warm")?;
    let plasma_core = config
        .plasma_core_rgb()
        .context("invalid plasma.color_core")?;
    let plasma_edge = config
        .plasma_edge_rgb()
        .context("invalid plasma.color_edge")?;
    Ok(RawRenderParams {
        width: config.render.width,
        height: config.render.height,
        frame_index,
        total_frames,
        seed: config.scene.seed,
        time_seconds: (frame_index as f32 / config.scene.fps) * config.simulation.motion_scale,
        duration_seconds: config.scene.duration,
        fps: config.scene.fps,
        quality: match config.render.quality {
            Quality::Preview => 0,
            Quality::Final => 1,
            Quality::Ultra => 2,
        },
        field_width: config.simulation.field_width,
        field_height: config.simulation.field_height,
        substeps: config.simulation.substeps,
        velocity_decay: config.simulation.velocity_decay,
        vorticity: config.simulation.vorticity,
        buoyancy: config.simulation.buoyancy,
        turbulence: config.simulation.turbulence,
        swirl: config.simulation.swirl,
        advection_dissipation: config.simulation.advection_dissipation,
        plasma_emission_strength: config.plasma.emission_strength,
        plasma_filament_density: config.plasma.filament_density,
        plasma_core_temperature: config.plasma.core_temperature,
        plasma_edge_wrapping: config.plasma.edge_wrapping,
        plasma_color_core: [plasma_core.r, plasma_core.g, plasma_core.b],
        plasma_color_edge: [plasma_edge.r, plasma_edge.g, plasma_edge.b],
        particles_count: config.particles.count,
        particles_size_min: config.particles.size_min,
        particles_size_max: config.particles.size_max,
        particles_lifetime: config.particles.lifetime,
        particles_flow_coupling: config.particles.flow_coupling,
        particles_ember_brightness: config.particles.ember_brightness,
        particles_gravity: config.particles.gravity,
        particles_drag: config.particles.drag,
        particles_thermal_lift: config.particles.thermal_lift,
        particles_jitter: config.particles.jitter,
        smoke_density: config.smoke.density,
        smoke_scattering: config.smoke.scattering,
        smoke_absorption: config.smoke.absorption,
        smoke_color_cool: [cool.r, cool.g, cool.b],
        smoke_color_warm: [warm.r, warm.g, warm.b],
        bloom_threshold: config.bloom.threshold,
        bloom_strength: config.bloom.strength,
        bloom_radius: config.bloom.radius,
        bloom_pyramid_levels: config.bloom.pyramid_levels,
        grade_exposure: config.grade.exposure,
        grade_contrast: config.grade.contrast,
        grade_saturation: config.grade.saturation,
        grade_black_point: config.grade.black_point,
        grade_vignette: config.grade.vignette,
        grade_grain: config.grade.grain,
    })
}

fn c_string<const N: usize>(bytes: &[c_char; N]) -> String {
    unsafe { CStr::from_ptr(bytes.as_ptr()) }
        .to_string_lossy()
        .trim()
        .to_string()
}

fn cuda_version(version: c_int) -> String {
    if version <= 0 {
        return "unknown".to_string();
    }
    let major = version / 1000;
    let minor = (version % 1000) / 10;
    format!("{major}.{minor}")
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, thread};

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
