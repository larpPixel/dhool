use dhool_core::FrameTiming;
use eframe::egui::{self, Ui};

pub fn draw(
    ui: &mut Ui,
    frame_index: &mut u32,
    frame_count: u32,
    fps: f32,
    playing: &mut bool,
    loop_playback: &mut bool,
    max_gpu: &mut bool,
    target_preview_fps: &mut f32,
    timing: Option<&FrameTiming>,
    preview_scale: f32,
    in_flight: usize,
    pipeline_depth: usize,
) -> bool {
    let mut changed = false;
    let last_frame = frame_count.saturating_sub(1);
    *frame_index = (*frame_index).min(last_frame);

    ui.horizontal(|ui| {
        let play_label = if *playing { "Pause" } else { "Play" };
        if ui.button(play_label).clicked() {
            *playing = !*playing;
            changed = true;
        }
        changed |= ui.checkbox(loop_playback, "Loop").changed();
        changed |= ui.checkbox(max_gpu, "Max GPU").changed();
        changed |= ui
            .add_enabled(
                !*max_gpu,
                egui::Slider::new(target_preview_fps, 1.0..=240.0).text("Preview fps"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(frame_index, 0..=last_frame)
                    .text("Frame")
                    .show_value(false),
            )
            .changed();

        let current_seconds = *frame_index as f32 / fps.max(1.0);
        ui.label(format!("{current_seconds:.2}s"));
        ui.separator();
        ui.label(format!("Frame: {}/{}", *frame_index + 1, frame_count));

        if let Some(timing) = timing {
            let render_fps = if timing.gpu_ms > 0.0 {
                1000.0 / timing.gpu_ms
            } else {
                0.0
            };
            ui.label(format!(
                "GPU: {:.1} ms  Readback: {:.1} ms  {:.1} fps",
                timing.gpu_ms, timing.readback_ms, render_fps
            ));
        } else {
            ui.label("GPU: waiting");
        }

        ui.label(format!("Scale: {preview_scale:.2}"));
        ui.label(format!("Queue: {in_flight}/{pipeline_depth}"));
    });

    changed
}
