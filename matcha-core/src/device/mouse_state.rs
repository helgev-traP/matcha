use super::button_state::ButtonState;
use crate::{
    device::mouse::MouseButton,
    events::{ConcreteEvent, Event, MouseEvent},
};
use std::time::{Duration, Instant};

/// Manages the mouse state to detect complex gestures like clicks, drags, and long presses
/// from raw mouse input events.
pub struct MouseState {
    /// The maximum duration between clicks to be considered a combo (e.g., double-click).
    combo_duration: Duration,
    /// The duration a button must be held down to be considered a long press.
    long_press_duration: Duration,

    /// The current position of the cursor.
    position: [f32; 2],

    primary: ButtonState,
    primary_dragging_from: Option<[f32; 2]>,

    secondary: ButtonState,
    secondary_dragging_from: Option<[f32; 2]>,

    middle: ButtonState,
    middle_dragging_from: Option<[f32; 2]>,
}

impl MouseState {
    /// A helper function to create a `ConcreteEvent::MouseEvent`.
    fn new_mouse_event(&self, event: MouseEvent) -> ConcreteEvent {
        ConcreteEvent::MouseEvent {
            current_position: self.position,
            dragging_primary: self.primary_dragging_from,
            dragging_secondary: self.secondary_dragging_from,
            dragging_middle: self.middle_dragging_from,
            event,
        }
    }

    /// Creates a new `MouseState`.
    ///
    /// # Arguments
    ///
    /// * `combo_duration` - The time in seconds to detect a combo click.
    /// * `long_press_duration` - The time in seconds to detect a long press.
    ///
    /// Returns `None` if `combo_duration` is greater than `long_press_duration`.
    pub fn new(combo_duration: Duration, long_press_duration: Duration) -> Option<Self> {
        if combo_duration <= long_press_duration {
            Some(Self {
                combo_duration,
                long_press_duration,
                position: [0.0, 0.0],
                primary: ButtonState::default(),
                primary_dragging_from: None,
                secondary: ButtonState::default(),
                secondary_dragging_from: None,
                middle: ButtonState::default(),
                middle_dragging_from: None,
            })
        } else {
            None
        }
    }

    /// Detects long presses for all mouse buttons.
    ///
    /// This method should be called on every frame update. It checks if any button has been
    /// held down for the `long_press_duration` without being dragged, and if so, generates
    /// a `LongPressed` event.
    pub fn long_pressing_detection(&mut self, now: Instant) -> Vec<Event> {
        let mut events = Vec::new();

        if self.primary_dragging_from.is_none() {
            if let Some(click_state) = self.primary.detect_long_press(now, self.long_press_duration)
            {
                let event = self.new_mouse_event(MouseEvent::Click {
                    click_state,
                    button: MouseButton::Primary,
                });
                events.push(Event::new(event));
            }
        }

        if self.secondary_dragging_from.is_none() {
            if let Some(click_state) =
                self.secondary
                    .detect_long_press(now, self.long_press_duration)
            {
                let event = self.new_mouse_event(MouseEvent::Click {
                    click_state,
                    button: MouseButton::Secondary,
                });
                events.push(Event::new(event));
            }
        }

        if self.middle_dragging_from.is_none() {
            if let Some(click_state) = self.middle.detect_long_press(now, self.long_press_duration)
            {
                let event = self.new_mouse_event(MouseEvent::Click {
                    click_state,
                    button: MouseButton::Middle,
                });
                events.push(Event::new(event));
            }
        }

        events
    }

    /// Handles a mouse button press event.
    ///
    /// It updates the click combo count and status for the given button and generates a `Pressed` event.
    pub fn button_pressed(&mut self, now: Instant, button: MouseButton) -> Event {
        let combo_duration = self.combo_duration;
        let (button_state, _) = self.get_mut_button_state(button);
        let click_state = button_state.press(now, combo_duration);

        let event = self.new_mouse_event(MouseEvent::Click { click_state, button });
        Event::new(event)
    }

    /// Handles a mouse move event.
    ///
    /// Updates the cursor position and detects the start of a drag for any pressed buttons.
    /// It generates a `CursorMove` event containing the drag state.
    pub fn mouse_move(&mut self, position: [f32; 2]) -> Event {
        if self.primary.is_pressed() && self.primary_dragging_from.is_none() {
            self.primary_dragging_from = Some(self.position);
        }
        if self.secondary.is_pressed() && self.secondary_dragging_from.is_none() {
            self.secondary_dragging_from = Some(self.position);
        }
        if self.middle.is_pressed() && self.middle_dragging_from.is_none() {
            self.middle_dragging_from = Some(self.position);
        }

        self.position = position;

        let event = self.new_mouse_event(MouseEvent::Move);
        Event::new(event)
    }

    /// Handles a mouse button release event.
    ///
    /// Resets the click status and drag state for the given button and generates a `Released` event.
    pub fn button_released(&mut self, button: MouseButton) -> Event {
        let (button_state, dragging_from) = self.get_mut_button_state(button);
        let click_state = button_state.release();
        *dragging_from = None;

        let event = self.new_mouse_event(MouseEvent::Click { click_state, button });
        Event::new(event)
    }

    /// Generates a `CursorEntered` event.
    pub fn cursor_entered(&self) -> Event {
        let event = self.new_mouse_event(MouseEvent::Entered);
        Event::new(event)
    }

    /// Generates a `CursorLeft` event.
    pub fn cursor_left(&self) -> Event {
        let event = self.new_mouse_event(MouseEvent::Left);
        Event::new(event)
    }

    /// Generates a `MouseScroll` event.
    pub fn mouse_scroll(&self, delta: [f32; 2]) -> Event {
        let event = self.new_mouse_event(MouseEvent::Scroll { delta });
        Event::new(event)
    }

    /// Helper function to get mutable references to the state for a specific button.
    fn get_mut_button_state(
        &mut self,
        button: MouseButton,
    ) -> (&mut ButtonState, &mut Option<[f32; 2]>) {
        match button {
            MouseButton::Primary => (&mut self.primary, &mut self.primary_dragging_from),
            MouseButton::Secondary => (&mut self.secondary, &mut self.secondary_dragging_from),
            MouseButton::Middle => (&mut self.middle, &mut self.middle_dragging_from),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::events::ElementState;
    use super::*;

    const COMBO_DURATION: Duration = Duration::from_millis(200);
    const LONG_PRESS_DURATION: Duration = Duration::from_millis(1000);

    #[test]
    fn click_and_long_press() {
        let mut mouse_state = MouseState::new(COMBO_DURATION, LONG_PRESS_DURATION).unwrap();
        let mut now = Instant::now();

        for b in [
            MouseButton::Primary,
            MouseButton::Secondary,
            MouseButton::Middle,
        ] {
            // --- Test combo clicks ---
            for i in 0..10 {
                let event = mouse_state.button_pressed(now, b);
                assert_eq!(
                    *event.raw_event(),
                    mouse_state.new_mouse_event(MouseEvent::Click {
                        click_state: ElementState::Pressed(i + 1),
                        button: b
                    })
                );

                let event = mouse_state.button_released(b);
                assert_eq!(
                    *event.raw_event(),
                    mouse_state.new_mouse_event(MouseEvent::Click {
                        click_state: ElementState::Released(i + 1),
                        button: b
                    })
                );

                // Elapse time within combo duration
                now += Duration::from_millis(100);
            }

            // --- Test long press ---
            // Elapse time to reset combo
            now += Duration::from_secs(1);

            let _ = mouse_state.button_pressed(now, b);

            // Elapse time for long press
            now += LONG_PRESS_DURATION;

            let events = mouse_state.long_pressing_detection(now);
            assert_eq!(
                *events[0].raw_event(),
                mouse_state.new_mouse_event(MouseEvent::Click {
                    click_state: ElementState::LongPressed(1), // Combo is reset
                    button: b
                })
            );

            let event = mouse_state.button_released(b);
            assert_eq!(
                *event.raw_event(),
                mouse_state.new_mouse_event(MouseEvent::Click {
                    click_state: ElementState::Released(1),
                    button: b
                })
            );
        }
    }

    #[test]
    fn move_and_drag() {
        let mut mouse_state = MouseState::new(COMBO_DURATION, LONG_PRESS_DURATION).unwrap();
        let mut now = Instant::now();

        // --- Test single button dragging ---
        for b in [
            MouseButton::Primary,
            MouseButton::Secondary,
            MouseButton::Middle,
        ] {
            let event = mouse_state.mouse_move([0.0, 0.0]);
            assert_eq!(
                *event.raw_event(),
                mouse_state.new_mouse_event(MouseEvent::Move)
            );

            let _ = mouse_state.button_pressed(now, b);
            now += Duration::from_millis(10);

            let event = mouse_state.mouse_move([1.0, 1.0]);
            let expected_event = ConcreteEvent::MouseEvent {
                current_position: [1.0, 1.0],
                dragging_primary: if b == MouseButton::Primary {
                    Some([0.0, 0.0])
                } else {
                    None
                },
                dragging_secondary: if b == MouseButton::Secondary {
                    Some([0.0, 0.0])
                } else {
                    None
                },
                dragging_middle: if b == MouseButton::Middle {
                    Some([0.0, 0.0])
                } else {
                    None
                },
                event: MouseEvent::Move,
            };
            assert_eq!(*event.raw_event(), expected_event);

            // Elapse time for long press, but it shouldn't trigger because we are dragging
            now += LONG_PRESS_DURATION;

            let events = mouse_state.long_pressing_detection(now);
            assert_eq!(events.len(), 0);

            let event = mouse_state.button_released(b);
            assert_eq!(
                *event.raw_event(),
                mouse_state.new_mouse_event(MouseEvent::Click {
                    click_state: ElementState::Released(1),
                    button: b
                })
            );
            now += Duration::from_secs(1); // Reset for next loop
        }

        // --- Test multiple button dragging ---
        {
            let _ = mouse_state.mouse_move([0.0, 0.0]);
            let _ = mouse_state.button_pressed(now, MouseButton::Primary);
            now += Duration::from_millis(10);

            let event = mouse_state.mouse_move([1.0, 1.0]);
            let expected_event = ConcreteEvent::MouseEvent {
                current_position: [1.0, 1.0],
                dragging_primary: Some([0.0, 0.0]),
                dragging_secondary: None,
                dragging_middle: None,
                event: MouseEvent::Move,
            };
            assert_eq!(*event.raw_event(), expected_event);

            let _ = mouse_state.button_pressed(now, MouseButton::Secondary);
            now += Duration::from_millis(10);

            let event = mouse_state.mouse_move([2.0, 2.0]);
            let expected_event = ConcreteEvent::MouseEvent {
                current_position: [2.0, 2.0],
                dragging_primary: Some([0.0, 0.0]),
                dragging_secondary: Some([1.0, 1.0]),
                dragging_middle: None,
                event: MouseEvent::Move,
            };
            assert_eq!(*event.raw_event(), expected_event);

            let _ = mouse_state.button_pressed(now, MouseButton::Middle);
            now += Duration::from_millis(10);

            let event = mouse_state.mouse_move([3.0, 3.0]);
            let expected_event = ConcreteEvent::MouseEvent {
                current_position: [3.0, 3.0],
                dragging_primary: Some([0.0, 0.0]),
                dragging_secondary: Some([1.0, 1.0]),
                dragging_middle: Some([2.0, 2.0]),
                event: MouseEvent::Move,
            };
            assert_eq!(*event.raw_event(), expected_event);
        }
    }
}
