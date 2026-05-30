use dhool_core::FrameTiming;
use eframe::egui::{self, Ui, Vec2};

const TIMELINE_BUTTON: Vec2 = Vec2::new(64.0, 24.0);
const FRAME_SLIDER_WIDTH: f32 = 220.0;
const FPS_SLIDER_WIDTH: f32 = 150.0;

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

    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(8.0, 4.0);
        let play_label = if *playing { "Pause" } else { "Play" };
        if ui
            .add(egui::Button::new(play_label).min_size(TIMELINE_BUTTON))
            .clicked()
        {
            *playing = !*playing;
            changed = true;
        }
        changed |= ui.checkbox(loop_playback, "Loop").changed();
        changed |= ui.checkbox(max_gpu, "Max GPU").changed();
        changed |= ui
            .add_enabled_ui(!*max_gpu, |ui| {
                ui.add_sized(
                    [FPS_SLIDER_WIDTH, 22.0],
                    egui::Slider::new(target_preview_fps, 1.0..=240.0).text("Preview fps"),
                )
                .changed()
            })
            .inner;
        changed |= ui
            .add_sized(
                [FRAME_SLIDER_WIDTH, 22.0],
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
