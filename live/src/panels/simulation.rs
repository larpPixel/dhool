use dhool_config::{
    Config, MOTION_SCALE_MAX, MOTION_SCALE_MIN, RENDER_DIMENSION_MAX,
    SIMULATION_FIELD_DIMENSION_MIN, SIMULATION_FORCE_MAX, SIMULATION_FORCE_MIN,
    SIMULATION_SUBSTEPS_MIN, SIMULATION_SUBSTEPS_UI_MAX, UNIT_INTERVAL_MAX, UNIT_INTERVAL_MIN,
};
use eframe::egui::{self, DragValue, Ui};

pub(super) fn simulation_panel(ui: &mut Ui, config: &mut Config) -> bool {
    let mut changed = false;
    egui::CollapsingHeader::new("Simulation")
        .default_open(true)
        .show(ui, |ui| {
            changed |= ui
                .add(
                    DragValue::new(&mut config.simulation.field_width)
                        .range(SIMULATION_FIELD_DIMENSION_MIN..=RENDER_DIMENSION_MAX)
                        .speed(16.0)
                        .prefix("field width "),
                )
                .changed();
            changed |= ui
                .add(
                    DragValue::new(&mut config.simulation.field_height)
                        .range(SIMULATION_FIELD_DIMENSION_MIN..=RENDER_DIMENSION_MAX)
                        .speed(16.0)
                        .prefix("field height "),
                )
                .changed();
            changed |= ui
                .add(
                    DragValue::new(&mut config.simulation.substeps)
                        .range(SIMULATION_SUBSTEPS_MIN..=SIMULATION_SUBSTEPS_UI_MAX)
                        .speed(1.0)
                        .prefix("substeps "),
                )
                .changed();
            changed |= ui
                .add(
                    egui::Slider::new(
                        &mut config.simulation.motion_scale,
                        MOTION_SCALE_MIN..=MOTION_SCALE_MAX,
                    )
                    .text("motion scale"),
                )
                .changed();
            changed |= ui
                .add(
                    egui::Slider::new(
                        &mut config.simulation.velocity_decay,
                        UNIT_INTERVAL_MIN..=UNIT_INTERVAL_MAX,
                    )
                    .text("velocity decay"),
                )
                .changed();
            changed |= ui
                .add(
                    egui::Slider::new(
                        &mut config.simulation.vorticity,
                        SIMULATION_FORCE_MIN..=SIMULATION_FORCE_MAX,
                    )
                    .text("vorticity"),
                )
                .changed();
            changed |= ui
                .add(
                    egui::Slider::new(
                        &mut config.simulation.buoyancy,
                        SIMULATION_FORCE_MIN..=SIMULATION_FORCE_MAX,
                    )
                    .text("buoyancy"),
                )
                .changed();
            changed |= ui
                .add(
                    egui::Slider::new(
                        &mut config.simulation.turbulence,
                        SIMULATION_FORCE_MIN..=SIMULATION_FORCE_MAX,
                    )
                    .text("turbulence"),
                )
                .changed();
            changed |= ui
                .add(
                    egui::Slider::new(
                        &mut config.simulation.swirl,
                        SIMULATION_FORCE_MIN..=SIMULATION_FORCE_MAX,
                    )
                    .text("swirl"),
                )
                .changed();
            changed |= ui
                .add(
                    egui::Slider::new(
                        &mut config.simulation.advection_dissipation,
                        UNIT_INTERVAL_MIN..=UNIT_INTERVAL_MAX,
                    )
                    .text("advection dissipation"),
                )
                .changed();
        });
    changed
}
