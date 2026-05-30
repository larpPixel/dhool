use dhool_config::{
    Config, MOTION_SCALE_MAX, MOTION_SCALE_MIN, RENDER_DIMENSION_MAX,
    SIMULATION_FIELD_DIMENSION_MIN, SIMULATION_FORCE_MAX, SIMULATION_FORCE_MIN,
    SIMULATION_SUBSTEPS_MIN, SIMULATION_SUBSTEPS_UI_MAX, UNIT_INTERVAL_MAX, UNIT_INTERVAL_MIN,
};
use eframe::egui::Ui;

use crate::panels::form;

pub(super) fn simulation_panel(ui: &mut Ui, config: &mut Config) -> bool {
    let mut changed = false;
    form::section(ui, "Simulation", true, |ui| {
        changed |= form::drag_u32(
            ui,
            "Field width",
            &mut config.simulation.field_width,
            SIMULATION_FIELD_DIMENSION_MIN..=RENDER_DIMENSION_MAX,
            16.0,
        );
        changed |= form::drag_u32(
            ui,
            "Field height",
            &mut config.simulation.field_height,
            SIMULATION_FIELD_DIMENSION_MIN..=RENDER_DIMENSION_MAX,
            16.0,
        );
        changed |= form::drag_u32(
            ui,
            "Substeps",
            &mut config.simulation.substeps,
            SIMULATION_SUBSTEPS_MIN..=SIMULATION_SUBSTEPS_UI_MAX,
            1.0,
        );
        changed |= form::slider_f32(
            ui,
            "Motion scale",
            &mut config.simulation.motion_scale,
            MOTION_SCALE_MIN..=MOTION_SCALE_MAX,
        );
        changed |= form::slider_f32(
            ui,
            "Velocity decay",
            &mut config.simulation.velocity_decay,
            UNIT_INTERVAL_MIN..=UNIT_INTERVAL_MAX,
        );
        changed |= form::slider_f32(
            ui,
            "Vorticity",
            &mut config.simulation.vorticity,
            SIMULATION_FORCE_MIN..=SIMULATION_FORCE_MAX,
        );
        changed |= form::slider_f32(
            ui,
            "Buoyancy",
            &mut config.simulation.buoyancy,
            SIMULATION_FORCE_MIN..=SIMULATION_FORCE_MAX,
        );
        changed |= form::slider_f32(
            ui,
            "Turbulence",
            &mut config.simulation.turbulence,
            SIMULATION_FORCE_MIN..=SIMULATION_FORCE_MAX,
        );
        changed |= form::slider_f32(
            ui,
            "Swirl",
            &mut config.simulation.swirl,
            SIMULATION_FORCE_MIN..=SIMULATION_FORCE_MAX,
        );
        changed |= form::slider_f32(
            ui,
            "Dissipation",
            &mut config.simulation.advection_dissipation,
            UNIT_INTERVAL_MIN..=UNIT_INTERVAL_MAX,
        );
        changed
    })
}
