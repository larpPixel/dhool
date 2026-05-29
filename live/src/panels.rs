use dhool_config::{
    BackendKind, Config, ExportMode, InternalFormat, Quality, SequenceFormat, parse_hex_color,
};
use eframe::egui::{self, Color32, DragValue, Ui};

pub fn draw_all(ui: &mut Ui, config: &mut Config, preview_scale: &mut f32) -> bool {
    ui.spacing_mut().slider_width = 180.0;

    let mut changed = false;
    changed |= scene_panel(ui, config);
    changed |= render_panel(ui, config, preview_scale);
    changed |= backend_panel(ui, config);
    changed |= simulation_panel(ui, config);
    changed |= plasma_panel(ui, config);
    changed |= particles_panel(ui, config);
    changed |= smoke_panel(ui, config);
    changed |= bloom_panel(ui, config);
    changed |= grade_panel(ui, config);
    changed |= export_panel(ui, config);
    changed
}

fn scene_panel(ui: &mut Ui, config: &mut Config) -> bool {
    let mut changed = false;
    egui::CollapsingHeader::new("Scene")
        .default_open(true)
        .show(ui, |ui| {
            ui.label("name");
            changed |= ui.text_edit_singleline(&mut config.scene.name).changed();
            changed |= ui
                .add(
                    DragValue::new(&mut config.scene.seed)
                        .speed(1.0)
                        .prefix("seed "),
                )
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut config.scene.duration, 0.001..=24.0).text("duration"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut config.scene.fps, 1.0..=240.0).text("fps"))
                .changed();
        });
    changed
}

fn render_panel(ui: &mut Ui, config: &mut Config, preview_scale: &mut f32) -> bool {
    let mut changed = false;
    egui::CollapsingHeader::new("Render")
        .default_open(true)
        .show(ui, |ui| {
            changed |= ui
                .add(egui::Slider::new(preview_scale, 0.25..=1.0).text("preview scale"))
                .changed();
            changed |= ui
                .add(
                    DragValue::new(&mut config.render.width)
                        .range(16..=8192)
                        .speed(16.0)
                        .prefix("width "),
                )
                .changed();
            changed |= ui
                .add(
                    DragValue::new(&mut config.render.height)
                        .range(16..=8192)
                        .speed(16.0)
                        .prefix("height "),
                )
                .changed();
            changed |= quality_combo(ui, "quality", &mut config.render.quality);
            changed |= internal_format_combo(ui, &mut config.render.internal_format);
        });
    changed
}

fn backend_panel(ui: &mut Ui, config: &mut Config) -> bool {
    let mut changed = false;
    egui::CollapsingHeader::new("Backend").show(ui, |ui| {
        changed |= backend_combo(ui, "preferred", &mut config.backend.preferred);
        changed |= ui
            .checkbox(&mut config.backend.require_gpu, "require gpu")
            .changed();
    });
    changed
}

fn simulation_panel(ui: &mut Ui, config: &mut Config) -> bool {
    let mut changed = false;
    egui::CollapsingHeader::new("Simulation")
        .default_open(true)
        .show(ui, |ui| {
            changed |= ui
                .add(
                    DragValue::new(&mut config.simulation.field_width)
                        .range(16..=8192)
                        .speed(16.0)
                        .prefix("field width "),
                )
                .changed();
            changed |= ui
                .add(
                    DragValue::new(&mut config.simulation.field_height)
                        .range(16..=8192)
                        .speed(16.0)
                        .prefix("field height "),
                )
                .changed();
            changed |= ui
                .add(
                    DragValue::new(&mut config.simulation.substeps)
                        .range(1..=128)
                        .speed(1.0)
                        .prefix("substeps "),
                )
                .changed();
            changed |= ui
                .add(
                    egui::Slider::new(&mut config.simulation.motion_scale, 0.01..=4.0)
                        .text("motion scale"),
                )
                .changed();
            changed |= ui
                .add(
                    egui::Slider::new(&mut config.simulation.velocity_decay, 0.0..=1.0)
                        .text("velocity decay"),
                )
                .changed();
            changed |= ui
                .add(
                    egui::Slider::new(&mut config.simulation.vorticity, 0.0..=5.0)
                        .text("vorticity"),
                )
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut config.simulation.buoyancy, 0.0..=5.0).text("buoyancy"))
                .changed();
            changed |= ui
                .add(
                    egui::Slider::new(&mut config.simulation.turbulence, 0.0..=5.0)
                        .text("turbulence"),
                )
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut config.simulation.swirl, 0.0..=5.0).text("swirl"))
                .changed();
            changed |= ui
                .add(
                    egui::Slider::new(&mut config.simulation.advection_dissipation, 0.0..=1.0)
                        .text("advection dissipation"),
                )
                .changed();
        });
    changed
}

fn plasma_panel(ui: &mut Ui, config: &mut Config) -> bool {
    let mut changed = false;
    egui::CollapsingHeader::new("Plasma")
        .default_open(true)
        .show(ui, |ui| {
            changed |= ui
                .add(
                    egui::Slider::new(&mut config.plasma.emission_strength, 0.0..=32.0)
                        .text("emission strength"),
                )
                .changed();
            changed |= ui
                .add(
                    egui::Slider::new(&mut config.plasma.filament_density, 0.0..=8.0)
                        .text("filament density"),
                )
                .changed();
            changed |= ui
                .add(
                    egui::Slider::new(&mut config.plasma.core_temperature, 0.0..=4.0)
                        .text("core temperature"),
                )
                .changed();
            changed |= ui
                .add(
                    egui::Slider::new(&mut config.plasma.edge_wrapping, 0.0..=4.0)
                        .text("edge wrapping"),
                )
                .changed();
            changed |= hex_color_control(ui, "color core", &mut config.plasma.color_core);
            changed |= hex_color_control(ui, "color edge", &mut config.plasma.color_edge);
        });
    changed
}

fn particles_panel(ui: &mut Ui, config: &mut Config) -> bool {
    let mut changed = false;
    egui::CollapsingHeader::new("Particles").show(ui, |ui| {
        changed |= ui
            .add(
                DragValue::new(&mut config.particles.count)
                    .range(0..=2_000_000)
                    .speed(100.0)
                    .prefix("count "),
            )
            .changed();
        changed |= ui
            .add(egui::Slider::new(&mut config.particles.size_min, 0.01..=64.0).text("size min"))
            .changed();
        changed |= ui
            .add(egui::Slider::new(&mut config.particles.size_max, 0.01..=96.0).text("size max"))
            .changed();
        if config.particles.size_max < config.particles.size_min {
            config.particles.size_max = config.particles.size_min;
            changed = true;
        }
        changed |= ui
            .add(egui::Slider::new(&mut config.particles.lifetime, 0.01..=60.0).text("lifetime"))
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(&mut config.particles.flow_coupling, 0.0..=1.0)
                    .text("flow coupling"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(&mut config.particles.ember_brightness, 0.0..=32.0)
                    .text("ember brightness"),
            )
            .changed();
        changed |= ui
            .add(egui::Slider::new(&mut config.particles.gravity, 0.0..=2.0).text("gravity"))
            .changed();
        changed |= ui
            .add(egui::Slider::new(&mut config.particles.drag, 0.0..=20.0).text("drag"))
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(&mut config.particles.thermal_lift, 0.0..=4.0)
                    .text("thermal lift"),
            )
            .changed();
        changed |= ui
            .add(egui::Slider::new(&mut config.particles.jitter, 0.0..=2.0).text("jitter"))
            .changed();
    });
    changed
}

fn smoke_panel(ui: &mut Ui, config: &mut Config) -> bool {
    let mut changed = false;
    egui::CollapsingHeader::new("Smoke").show(ui, |ui| {
        changed |= ui
            .add(egui::Slider::new(&mut config.smoke.density, 0.0..=8.0).text("density"))
            .changed();
        changed |= ui
            .add(egui::Slider::new(&mut config.smoke.scattering, 0.0..=8.0).text("scattering"))
            .changed();
        changed |= ui
            .add(egui::Slider::new(&mut config.smoke.absorption, 0.0..=8.0).text("absorption"))
            .changed();
        changed |= hex_color_control(ui, "color cool", &mut config.smoke.color_cool);
        changed |= hex_color_control(ui, "color warm", &mut config.smoke.color_warm);
    });
    changed
}

fn bloom_panel(ui: &mut Ui, config: &mut Config) -> bool {
    let mut changed = false;
    egui::CollapsingHeader::new("Bloom").show(ui, |ui| {
        changed |= ui
            .add(egui::Slider::new(&mut config.bloom.threshold, 0.0..=64.0).text("threshold"))
            .changed();
        changed |= ui
            .add(egui::Slider::new(&mut config.bloom.strength, 0.0..=8.0).text("strength"))
            .changed();
        changed |= ui
            .add(egui::Slider::new(&mut config.bloom.radius, 0.0..=8.0).text("radius"))
            .changed();
        changed |= ui
            .add(
                DragValue::new(&mut config.bloom.pyramid_levels)
                    .range(1..=8)
                    .speed(1.0)
                    .prefix("pyramid levels "),
            )
            .changed();
    });
    changed
}

fn grade_panel(ui: &mut Ui, config: &mut Config) -> bool {
    let mut changed = false;
    egui::CollapsingHeader::new("Grade").show(ui, |ui| {
        changed |= ui
            .add(egui::Slider::new(&mut config.grade.exposure, -8.0..=8.0).text("exposure"))
            .changed();
        changed |= ui
            .add(egui::Slider::new(&mut config.grade.contrast, 0.0..=4.0).text("contrast"))
            .changed();
        changed |= ui
            .add(egui::Slider::new(&mut config.grade.saturation, 0.0..=4.0).text("saturation"))
            .changed();
        changed |= ui
            .add(egui::Slider::new(&mut config.grade.black_point, 0.0..=1.0).text("black point"))
            .changed();
        changed |= ui
            .add(egui::Slider::new(&mut config.grade.vignette, 0.0..=2.0).text("vignette"))
            .changed();
        changed |= ui
            .add(egui::Slider::new(&mut config.grade.grain, 0.0..=1.0).text("grain"))
            .changed();
    });
    changed
}

fn export_panel(ui: &mut Ui, config: &mut Config) -> bool {
    let mut changed = false;
    egui::CollapsingHeader::new("Export").show(ui, |ui| {
        changed |= export_mode_combo(ui, "mode", &mut config.export.mode);
        changed |= sequence_format_combo(ui, "sequence format", &mut config.export.sequence_format);
        ui.label("video codec");
        changed |= ui
            .text_edit_singleline(&mut config.export.video_codec)
            .changed();
        changed |= ui
            .add(
                DragValue::new(&mut config.export.crf)
                    .range(0..=51)
                    .speed(1.0)
                    .prefix("crf "),
            )
            .changed();
        ui.label("preset");
        changed |= ui.text_edit_singleline(&mut config.export.preset).changed();
        ui.label("output dir");
        changed |= ui
            .text_edit_singleline(&mut config.export.output_dir)
            .changed();
    });
    changed
}

fn quality_combo(ui: &mut Ui, label: &str, value: &mut Quality) -> bool {
    let mut changed = false;
    egui::ComboBox::from_label(label)
        .selected_text(quality_label(*value))
        .show_ui(ui, |ui| {
            changed |= ui
                .selectable_value(value, Quality::Preview, quality_label(Quality::Preview))
                .changed();
            changed |= ui
                .selectable_value(value, Quality::Final, quality_label(Quality::Final))
                .changed();
            changed |= ui
                .selectable_value(value, Quality::Ultra, quality_label(Quality::Ultra))
                .changed();
        });
    changed
}

fn backend_combo(ui: &mut Ui, label: &str, value: &mut BackendKind) -> bool {
    let mut changed = false;
    egui::ComboBox::from_label(label)
        .selected_text(backend_label(*value))
        .show_ui(ui, |ui| {
            changed |= ui
                .selectable_value(value, BackendKind::Cuda, backend_label(BackendKind::Cuda))
                .changed();
            changed |= ui
                .selectable_value(
                    value,
                    BackendKind::CpuReference,
                    backend_label(BackendKind::CpuReference),
                )
                .changed();
        });
    changed
}

fn internal_format_combo(ui: &mut Ui, value: &mut InternalFormat) -> bool {
    let mut changed = false;
    egui::ComboBox::from_label("internal format")
        .selected_text(internal_format_label(*value))
        .show_ui(ui, |ui| {
            changed |= ui
                .selectable_value(
                    value,
                    InternalFormat::Rgba16f,
                    internal_format_label(InternalFormat::Rgba16f),
                )
                .changed();
        });
    changed
}

fn export_mode_combo(ui: &mut Ui, label: &str, value: &mut ExportMode) -> bool {
    let mut changed = false;
    egui::ComboBox::from_label(label)
        .selected_text(export_mode_label(*value))
        .show_ui(ui, |ui| {
            changed |= ui
                .selectable_value(value, ExportMode::Pipe, export_mode_label(ExportMode::Pipe))
                .changed();
            changed |= ui
                .selectable_value(
                    value,
                    ExportMode::Sequence,
                    export_mode_label(ExportMode::Sequence),
                )
                .changed();
            changed |= ui
                .selectable_value(value, ExportMode::Both, export_mode_label(ExportMode::Both))
                .changed();
        });
    changed
}

fn sequence_format_combo(ui: &mut Ui, label: &str, value: &mut SequenceFormat) -> bool {
    let mut changed = false;
    egui::ComboBox::from_label(label)
        .selected_text(sequence_format_label(*value))
        .show_ui(ui, |ui| {
            changed |= ui
                .selectable_value(
                    value,
                    SequenceFormat::Png,
                    sequence_format_label(SequenceFormat::Png),
                )
                .changed();
            changed |= ui
                .selectable_value(
                    value,
                    SequenceFormat::Exr,
                    sequence_format_label(SequenceFormat::Exr),
                )
                .changed();
        });
    changed
}

fn hex_color_control(ui: &mut Ui, label: &str, value: &mut String) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        let mut color = hex_to_color32(value).unwrap_or(Color32::WHITE);
        ui.label(label);
        if egui::color_picker::color_edit_button_srgba(
            ui,
            &mut color,
            egui::color_picker::Alpha::Opaque,
        )
        .changed()
        {
            *value = color32_to_hex(color);
            changed = true;
        }
    });
    changed |= ui.text_edit_singleline(value).changed();
    changed
}

fn hex_to_color32(value: &str) -> Option<Color32> {
    let rgb = parse_hex_color(value).ok()?;
    Some(Color32::from_rgb(
        channel_to_u8(rgb.r),
        channel_to_u8(rgb.g),
        channel_to_u8(rgb.b),
    ))
}

fn channel_to_u8(value: f32) -> u8 {
    (value * 255.0).round().clamp(0.0, 255.0) as u8
}

fn color32_to_hex(color: Color32) -> String {
    format!("#{:02x}{:02x}{:02x}", color.r(), color.g(), color.b())
}

fn quality_label(value: Quality) -> &'static str {
    match value {
        Quality::Preview => "preview",
        Quality::Final => "final",
        Quality::Ultra => "ultra",
    }
}

fn backend_label(value: BackendKind) -> &'static str {
    match value {
        BackendKind::Cuda => "cuda",
        BackendKind::CpuReference => "cpu-reference",
    }
}

fn internal_format_label(value: InternalFormat) -> &'static str {
    match value {
        InternalFormat::Rgba16f => "rgba16f",
    }
}

fn export_mode_label(value: ExportMode) -> &'static str {
    match value {
        ExportMode::Pipe => "pipe",
        ExportMode::Sequence => "sequence",
        ExportMode::Both => "both",
    }
}

fn sequence_format_label(value: SequenceFormat) -> &'static str {
    match value {
        SequenceFormat::Png => "png",
        SequenceFormat::Exr => "exr",
    }
}
