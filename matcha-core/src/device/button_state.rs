use crate::events::ElementState;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(super) enum ClickStatus {
    #[default]
    Released,
    Pressed,
    LongPressed,
}

/// Manages the state of a single button to detect combos and long presses.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct ButtonState {
    pub(super) status: ClickStatus,
    last_clicked_at: Option<Instant>,
    click_combo: u32,
}

impl ButtonState {
    pub(super) fn is_pressed(&self) -> bool {
        self.status != ClickStatus::Released
    }

    /// Handles a button press, updating the combo count and status.
    pub(super) fn press(&mut self, now: Instant, combo_duration: Duration) -> ElementState {
        if let Some(last_clicked_at) = self.last_clicked_at {
            if now.duration_since(last_clicked_at) <= combo_duration {
                self.click_combo += 1;
            } else {
                self.click_combo = 1;
            }
        } else {
            self.click_combo = 1;
        }

        self.last_clicked_at = Some(now);
        self.status = ClickStatus::Pressed;

        ElementState::Pressed(self.click_combo)
    }

    /// Handles a button release, resetting the status.
    pub(super) fn release(&mut self) -> ElementState {
        self.status = ClickStatus::Released;
        ElementState::Released(self.click_combo)
    }

    /// Detects a long press. Returns the corresponding `ElementState` if a long press was detected.
    pub(super) fn detect_long_press(
        &mut self,
        now: Instant,
        long_press_duration: Duration,
    ) -> Option<ElementState> {
        if self.status == ClickStatus::Pressed {
            if let Some(last_clicked_at) = self.last_clicked_at {
                if now.duration_since(last_clicked_at) >= long_press_duration {
                    self.status = ClickStatus::LongPressed;
                    return Some(ElementState::LongPressed(self.click_combo));
                }
            }
        }
        None
    }
}
