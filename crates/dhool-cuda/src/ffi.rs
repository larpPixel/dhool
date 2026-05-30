use std::{
    ffi::CStr,
    os::raw::{c_char, c_int},
};

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct RawDeviceInfo {
    pub(crate) available: c_int,
    pub(crate) name: [c_char; 256],
    pub(crate) driver_version: c_int,
    pub(crate) runtime_version: c_int,
    pub(crate) global_memory_bytes: u64,
    pub(crate) multiprocessor_count: c_int,
    pub(crate) error_code: c_int,
    pub(crate) error_message: [c_char; 512],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct RawRenderReport {
    pub(crate) status: c_int,
    pub(crate) gpu_ms: f32,
    pub(crate) readback_ms: f32,
    pub(crate) error_code: c_int,
    pub(crate) error_message: [c_char; 512],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct RawRenderParams {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) frame_index: u32,
    pub(crate) total_frames: u32,
    pub(crate) seed: u64,
    pub(crate) time_seconds: f32,
    pub(crate) duration_seconds: f32,
    pub(crate) fps: f32,
    pub(crate) quality: u32,
    pub(crate) field_width: u32,
    pub(crate) field_height: u32,
    pub(crate) substeps: u32,
    pub(crate) velocity_decay: f32,
    pub(crate) vorticity: f32,
    pub(crate) buoyancy: f32,
    pub(crate) turbulence: f32,
    pub(crate) swirl: f32,
    pub(crate) advection_dissipation: f32,
    pub(crate) plasma_emission_strength: f32,
    pub(crate) plasma_filament_density: f32,
    pub(crate) plasma_core_temperature: f32,
    pub(crate) plasma_edge_wrapping: f32,
    pub(crate) plasma_color_core: [f32; 3],
    pub(crate) plasma_color_edge: [f32; 3],
    pub(crate) particles_count: u32,
    pub(crate) particles_size_min: f32,
    pub(crate) particles_size_max: f32,
    pub(crate) particles_lifetime: f32,
    pub(crate) particles_flow_coupling: f32,
    pub(crate) particles_ember_brightness: f32,
    pub(crate) particles_gravity: f32,
    pub(crate) particles_drag: f32,
    pub(crate) particles_thermal_lift: f32,
    pub(crate) particles_jitter: f32,
    pub(crate) smoke_density: f32,
    pub(crate) smoke_scattering: f32,
    pub(crate) smoke_absorption: f32,
    pub(crate) smoke_color_cool: [f32; 3],
    pub(crate) smoke_color_warm: [f32; 3],
    pub(crate) bloom_threshold: f32,
    pub(crate) bloom_strength: f32,
    pub(crate) bloom_radius: f32,
    pub(crate) bloom_pyramid_levels: u32,
    pub(crate) grade_exposure: f32,
    pub(crate) grade_contrast: f32,
    pub(crate) grade_saturation: f32,
    pub(crate) grade_black_point: f32,
    pub(crate) grade_vignette: f32,
    pub(crate) grade_grain: f32,
}

unsafe extern "C" {
    pub(crate) fn dhool_cuda_device_info(out: *mut RawDeviceInfo) -> c_int;
    pub(crate) fn dhool_cuda_render_rgba8(
        params: *const RawRenderParams,
        host_rgba: *mut u8,
        report: *mut RawRenderReport,
    ) -> c_int;
}

pub(crate) fn c_string<const N: usize>(bytes: &[c_char; N]) -> String {
    unsafe { CStr::from_ptr(bytes.as_ptr()) }
        .to_string_lossy()
        .trim()
        .to_string()
}

pub(crate) fn cuda_version(version: c_int) -> String {
    if version <= 0 {
        return "unknown".to_string();
    }
    let major = version / 1000;
    let minor = (version % 1000) / 10;
    format!("{major}.{minor}")
}
