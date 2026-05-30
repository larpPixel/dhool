use std::time::{Duration, Instant};

use crate::app::LiveApp;

impl LiveApp {
    pub(super) fn frame_count(&self) -> u32 {
        self.config.frame_count(None).max(1)
    }

    pub(super) fn advance_playback_realtime(&mut self) -> bool {
        if !self.playing {
            self.last_play_tick = Instant::now();
            return false;
        }

        let fps = self.config.scene.fps.max(1.0);
        let frame_period = Duration::from_secs_f32(1.0 / fps);
        let now = Instant::now();
        let elapsed = now.saturating_duration_since(self.last_play_tick);
        if elapsed < frame_period {
            return false;
        }

        let steps = (elapsed.as_secs_f32() / frame_period.as_secs_f32()).floor() as u32;
        self.last_play_tick = now;
        if steps == 0 {
            return false;
        }

        self.advance_frames(steps)
    }

    pub(super) fn reset_playback_queue(&mut self) {
        self.next_play_frame_index = self.frame_index.min(self.frame_count().saturating_sub(1));
        self.last_play_tick = Instant::now();
    }

    pub(super) fn advance_next_play_frame(&mut self, rendered_frame_index: u32, frame_count: u32) {
        let next = rendered_frame_index.saturating_add(1);
        if next >= frame_count {
            if self.loop_playback {
                self.next_play_frame_index = 0;
            } else {
                self.next_play_frame_index = frame_count.saturating_sub(1);
                self.playing = false;
            }
        } else {
            self.next_play_frame_index = next;
        }
    }

    fn advance_frames(&mut self, steps: u32) -> bool {
        if steps == 0 {
            return false;
        }

        let frame_count = self.frame_count();
        let next = self.frame_index.saturating_add(steps);
        if next >= frame_count {
            if self.loop_playback {
                self.frame_index = next % frame_count;
            } else {
                self.frame_index = frame_count.saturating_sub(1);
                self.playing = false;
            }
        } else {
            self.frame_index = next;
        }
        true
    }
}
