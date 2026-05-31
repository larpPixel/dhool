use dhool_core::FrameTiming;
use eframe::egui::{self, Align, DragValue, Layout, RichText, Ui, Vec2};

use crate::app::theme;

const TIMELINE_BUTTON: Vec2 = Vec2::new(72.0, 28.0);
const FRAME_SLIDER_WIDTH: f32 = 260.0;
const FRAME_SLIDER_COMPACT_WIDTH: f32 = 200.0;
const FPS_SLIDER_WIDTH: f32 = 120.0;
const FPS_SLIDER_COMPACT_WIDTH: f32 = 96.0;
const FPS_VALUE_WIDTH: f32 = 46.0;

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

    let compact = ui.available_width() < 1120.0;
    let frame_slider_width = if compact {
        FRAME_SLIDER_COMPACT_WIDTH
    } else {
        FRAME_SLIDER_WIDTH
    };
    let fps_slider_width = if compact {
        FPS_SLIDER_COMPACT_WIDTH
    } else {
        FPS_SLIDER_WIDTH
    };

    ui.spacing_mut().item_spacing = egui::vec2(9.0, 0.0);
    ui.horizontal_centered(|ui| {
        let play_label = if *playing { "Pause" } else { "Play" };
        if theme::button(
            ui,
            play_label,
            TIMELINE_BUTTON,
            true,
            theme::ButtonKind::Primary,
        )
        .clicked()
        {
            *playing = !*playing;
            changed = true;
        }
        changed |= ui.checkbox(loop_playback, "Loop").changed();
        changed |= ui.checkbox(max_gpu, "Max GPU").changed();
        if !*max_gpu {
            small_label(ui, "Preview fps");
            changed |= ui
                .add_sized(
                    [fps_slider_width, theme::CONTROL_HEIGHT],
                    egui::Slider::new(target_preview_fps, 1.0..=240.0).show_value(false),
                )
                .changed();
            changed |= ui
                .add_sized(
                    [FPS_VALUE_WIDTH, theme::CONTROL_HEIGHT],
                    DragValue::new(target_preview_fps)
                        .range(1.0..=240.0)
                        .speed(1.0)
                        .max_decimals(0),
                )
                .changed();
        } else if !compact {
            small_label(ui, "Max throughput");
        }
        small_label(ui, "Frame");
        changed |= ui
            .add_sized(
                [frame_slider_width, theme::CONTROL_HEIGHT],
                egui::Slider::new(frame_index, 0..=last_frame).show_value(false),
            )
            .changed();
        let current_seconds = *frame_index as f32 / fps.max(1.0);
        ui.label(
            RichText::new(format!("{current_seconds:.2}s"))
                .monospace()
                .color(theme::TEXT),
        );

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if !compact {
                metric_text(ui, &format!("Queue {in_flight}/{pipeline_depth}"));
                metric_text(ui, &format!("Scale {preview_scale:.2}"));
            }
            if let Some(timing) = timing {
                let render_fps = if timing.gpu_ms > 0.0 {
                    1000.0 / timing.gpu_ms
                } else {
                    0.0
                };
                let timing_label = if compact {
                    format!("GPU {:.1} ms  {:.1} fps", timing.gpu_ms, render_fps)
                } else {
                    format!(
                        "GPU {:.1} ms  Readback {:.1} ms  {:.1} fps",
                        timing.gpu_ms, timing.readback_ms, render_fps
                    )
                };
                metric_text(ui, &timing_label);
            } else {
                metric_text(ui, "GPU waiting");
            }
            metric_text(ui, &format!("Frame {}/{}", *frame_index + 1, frame_count));
        });
    });

    changed
}

fn metric_text(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).size(11.0).color(theme::TEXT_MUTED));
}

fn small_label(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).size(11.0).color(theme::TEXT_MUTED));
}
