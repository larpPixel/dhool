use anyhow::{Context, Result};
use dhool_config::{Config, Quality};

use crate::ffi::RawRenderParams;

pub(crate) fn params_from_config(
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
