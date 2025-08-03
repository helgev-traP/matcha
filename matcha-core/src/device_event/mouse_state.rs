use super::{ButtonState, DeviceEvent, DeviceEventData, MouseInput, MouseLogicalButton};

use std::time::{Duration, Instant};
use winit::{
    dpi::PhysicalPosition,
    event::{MouseButton as WinitMouseButton, MouseScrollDelta},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MousePrimaryButton {
    #[default]
    Left,
    Right,
}

/// Manages the mouse state to detect complex gestures like clicks, drags, and long presses
/// from raw mouse input events.
pub struct MouseState {
    /// The maximum duration between clicks to be considered a combo (e.g., double-click).
    combo_duration: Duration,
    /// The duration a button must be held down to be considered a long press.
    long_press_duration: Duration,

    /// The current position of the cursor.
    position: [f32; 2],

    /// The physical button assigned as the primary button.
    primary_button: MousePrimaryButton,

    pixel_per_line: f32,

    // State for each logical button
    primary: ButtonState,
    primary_dragging_from: Option<[f32; 2]>,
    secondary: ButtonState,
    secondary_dragging_from: Option<[f32; 2]>,
    middle: ButtonState,
    middle_dragging_from: Option<[f32; 2]>,
    back: ButtonState,
    back_dragging_from: Option<[f32; 2]>,
    forward: ButtonState,
    forward_dragging_from: Option<[f32; 2]>,
}

impl MouseState {
    /// Creates a new `MouseState`.
    ///
    /// # Arguments
    ///
    /// * `combo_duration` - The time in seconds to detect a combo click.
    /// * `long_press_duration` - The time in seconds to detect a long press.
    /// * `primary_button` - The physical button to be treated as the primary button (e.g., `WinitMouseButton::Left`).
    ///
    /// Returns `None` if `combo_duration` is greater than `long_press_duration`.
    pub fn new(
        combo_duration: Duration,
        long_press_duration: Duration,
        primary_button: MousePrimaryButton,
        pixel_per_line: f32,
    ) -> Option<Self> {
        if combo_duration <= long_press_duration {
            Some(Self {
                combo_duration,
                long_press_duration,
                position: [0.0, 0.0],
                primary_button,
                pixel_per_line,
                primary: ButtonState::default(),
                primary_dragging_from: None,
                secondary: ButtonState::default(),
                secondary_dragging_from: None,
                middle: ButtonState::default(),
                middle_dragging_from: None,
                back: ButtonState::default(),
                back_dragging_from: None,
                forward: ButtonState::default(),
                forward_dragging_from: None,
            })
        } else {
            None
        }
    }

    pub fn set_primary_button(&mut self, primary_button: MousePrimaryButton) {
        self.primary_button = primary_button;
    }

    pub fn primary_button(&self) -> MousePrimaryButton {
        self.primary_button
    }

    pub fn set_scroll_pixel_per_line(&mut self, pixel: f32) {
        self.pixel_per_line = pixel;
    }

    pub fn scroll_pixel_per_line(&self) -> f32 {
        self.pixel_per_line
    }
}

impl MouseState {
    /// Handles a mouse move event.
    ///
    /// Updates the cursor position and detects the start of a drag for any pressed buttons.
    /// It generates a `CursorMove` event containing the drag state.
    pub fn cursor_moved(&mut self, position: PhysicalPosition<f64>) -> DeviceEvent {
        let prev_position = self.position;
        self.position = [position.x as f32, position.y as f32];

        if self.primary.is_pressed() && self.primary_dragging_from.is_none() {
            self.primary_dragging_from = Some(prev_position);
        }
        if self.secondary.is_pressed() && self.secondary_dragging_from.is_none() {
            self.secondary_dragging_from = Some(prev_position);
        }
        if self.middle.is_pressed() && self.middle_dragging_from.is_none() {
            self.middle_dragging_from = Some(prev_position);
        }
        if self.back.is_pressed() && self.back_dragging_from.is_none() {
            self.back_dragging_from = Some(prev_position);
        }
        if self.forward.is_pressed() && self.forward_dragging_from.is_none() {
            self.forward_dragging_from = Some(prev_position);
        }

        let event = Self::new_mouse_event(
            self.position,
            self.primary_dragging_from,
            self.secondary_dragging_from,
            self.middle_dragging_from,
            None,
        );
        DeviceEvent::new(event)
    }

    /// Generates a `CursorEntered` event.
    pub fn cursor_entered(&self) -> DeviceEvent {
        let event = Self::new_mouse_event(
            self.position,
            self.primary_dragging_from,
            self.secondary_dragging_from,
            self.middle_dragging_from,
            Some(MouseInput::Entered),
        );
        DeviceEvent::new(event)
    }

    /// Generates a `CursorLeft` event.
    pub fn cursor_left(&self) -> DeviceEvent {
        let event = Self::new_mouse_event(
            self.position,
            self.primary_dragging_from,
            self.secondary_dragging_from,
            self.middle_dragging_from,
            Some(MouseInput::Left),
        );
        DeviceEvent::new(event)
    }

    /// Generates a `MouseScroll` event.
    pub fn mouse_wheel(&self, delta: MouseScrollDelta) -> DeviceEvent {
        let delta = match delta {
            MouseScrollDelta::LineDelta(x, y) => [x * self.pixel_per_line, y * self.pixel_per_line],
            MouseScrollDelta::PixelDelta(PhysicalPosition { x, y }) => [x as f32, y as f32],
        };

        let event = Self::new_mouse_event(
            self.position,
            self.primary_dragging_from,
            self.secondary_dragging_from,
            self.middle_dragging_from,
            Some(MouseInput::Scroll { delta }),
        );
        DeviceEvent::new(event)
    }

    pub fn mouse_input(
        &mut self,
        physical_button: WinitMouseButton,
        state: winit::event::ElementState,
    ) -> Option<DeviceEvent> {
        match state {
            winit::event::ElementState::Pressed => self.button_pressed(physical_button),
            winit::event::ElementState::Released => self.button_released(physical_button),
        }
    }

    /// Handles a mouse button press event.
    ///
    /// It updates the click combo count and status for the given button and generates a `Pressed` event.
    fn button_pressed(&mut self, physical_button: WinitMouseButton) -> Option<DeviceEvent> {
        let now = Instant::now();

        let logical_button = self.to_logical_button(physical_button)?;
        let combo_duration = self.combo_duration;
        let (button_state, _) = self.get_mut_button_state(logical_button);
        let click_state = button_state.press(now, combo_duration);

        let event = Self::new_mouse_event(
            self.position,
            self.primary_dragging_from,
            self.secondary_dragging_from,
            self.middle_dragging_from,
            Some(MouseInput::Click {
                click_state,
                button: logical_button,
            }),
        );
        Some(DeviceEvent::new(event))
    }

    /// Handles a mouse button release event.
    ///
    /// Resets the click status and drag state for the given button and generates a `Released` event.
    fn button_released(&mut self, physical_button: WinitMouseButton) -> Option<DeviceEvent> {
        let logical_button = self.to_logical_button(physical_button)?;
        let (button_state, dragging_from) = self.get_mut_button_state(logical_button);
        let click_state = button_state.release();
        *dragging_from = None;

        let event = Self::new_mouse_event(
            self.position,
            self.primary_dragging_from,
            self.secondary_dragging_from,
            self.middle_dragging_from,
            Some(MouseInput::Click {
                click_state,
                button: logical_button,
            }),
        );
        Some(DeviceEvent::new(event))
    }

    /// Detects long presses for all mouse buttons.
    ///
    /// This method should be called on every frame update. It checks if any button has been
    /// held down for the `long_press_duration` without being dragged, and if so, generates
    /// a `LongPressed` event.
    pub fn long_pressing_detection(&mut self) -> Vec<DeviceEvent> {
        let now = Instant::now();

        let mut events = Vec::new();
        let buttons = [
            (
                MouseLogicalButton::Primary,
                &mut self.primary,
                self.primary_dragging_from,
            ),
            (
                MouseLogicalButton::Secondary,
                &mut self.secondary,
                self.secondary_dragging_from,
            ),
            (
                MouseLogicalButton::Middle,
                &mut self.middle,
                self.middle_dragging_from,
            ),
            (
                MouseLogicalButton::Back,
                &mut self.back,
                self.back_dragging_from,
            ),
            (
                MouseLogicalButton::Forward,
                &mut self.forward,
                self.forward_dragging_from,
            ),
        ];

        let current_position = self.position;
        let dragging_primary = self.primary_dragging_from;
        let dragging_secondary = self.secondary_dragging_from;
        let dragging_middle = self.middle_dragging_from;

        for (logical_button, button_state, dragging_from) in buttons {
            if dragging_from.is_none() {
                if let Some(click_state) =
                    button_state.detect_long_press(now, self.long_press_duration)
                {
                    let event = Self::new_mouse_event(
                        current_position,
                        dragging_primary,
                        dragging_secondary,
                        dragging_middle,
                        Some(MouseInput::Click {
                            click_state,
                            button: logical_button,
                        }),
                    );
                    events.push(DeviceEvent::new(event));
                }
            }
        }
        events
    }
}

// helper methods
impl MouseState {
    pub fn position(&self) -> [f32; 2] {
        self.position
    }

    /// Converts a physical `WinitMouseButton` to a `LogicalMouseButton` based on the primary button setting.
    fn to_logical_button(&self, physical_button: WinitMouseButton) -> Option<MouseLogicalButton> {
        match physical_button {
            WinitMouseButton::Left => {
                if self.primary_button == MousePrimaryButton::Left {
                    Some(MouseLogicalButton::Primary)
                } else {
                    Some(MouseLogicalButton::Secondary)
                }
            }
            WinitMouseButton::Right => {
                if self.primary_button == MousePrimaryButton::Left {
                    Some(MouseLogicalButton::Secondary)
                } else {
                    Some(MouseLogicalButton::Primary)
                }
            }
            WinitMouseButton::Middle => Some(MouseLogicalButton::Middle),
            WinitMouseButton::Back => Some(MouseLogicalButton::Back),
            WinitMouseButton::Forward => Some(MouseLogicalButton::Forward),
            _ => None,
        }
    }

    /// Gets mutable references to the state for a specific logical button.
    fn get_mut_button_state(
        &mut self,
        button: MouseLogicalButton,
    ) -> (&mut ButtonState, &mut Option<[f32; 2]>) {
        match button {
            MouseLogicalButton::Primary => (&mut self.primary, &mut self.primary_dragging_from),
            MouseLogicalButton::Secondary => {
                (&mut self.secondary, &mut self.secondary_dragging_from)
            }

            MouseLogicalButton::Middle => (&mut self.middle, &mut self.middle_dragging_from),
            MouseLogicalButton::Back => (&mut self.back, &mut self.back_dragging_from),
            MouseLogicalButton::Forward => (&mut self.forward, &mut self.forward_dragging_from),
        }
    }

    /// A helper function to create a `ConcreteEvent::MouseEvent`.
    fn new_mouse_event(
        current_position: [f32; 2],
        dragging_primary: Option<[f32; 2]>,
        dragging_secondary: Option<[f32; 2]>,
        dragging_middle: Option<[f32; 2]>,
        event: Option<MouseInput>,
    ) -> DeviceEventData {
        DeviceEventData::MouseEvent {
            current_position,
            dragging_primary,
            dragging_secondary,
            dragging_middle,
            event,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::super::ElementState;
    use super::*;
    use std::thread;
    use winit::event::ElementState as WinitElementState;

    const COMBO_DURATION: Duration = Duration::from_millis(200);
    const LONG_PRESS_DURATION: Duration = Duration::from_millis(500);
    const PIXEL_PER_LINE: f32 = 40.0;

    #[test]
    fn click_and_long_press() {
        let mut mouse_state = MouseState::new(
            COMBO_DURATION,
            LONG_PRESS_DURATION,
            MousePrimaryButton::Left,
            PIXEL_PER_LINE,
        )
        .unwrap();

        let physical_buttons = [
            WinitMouseButton::Left,
            WinitMouseButton::Right,
            WinitMouseButton::Middle,
            WinitMouseButton::Back,
            WinitMouseButton::Forward,
        ];

        for &b in &physical_buttons {
            let logical_b = mouse_state.to_logical_button(b).unwrap();

            // --- Test single click ---
            let event = mouse_state
                .mouse_input(b, WinitElementState::Pressed)
                .unwrap();
            let expected = MouseState::new_mouse_event(
                [0.0, 0.0],
                None,
                None,
                None,
                Some(MouseInput::Click {
                    click_state: ElementState::Pressed(1),
                    button: logical_b,
                }),
            );
            assert_eq!(*event.raw_event(), expected);

            thread::sleep(Duration::from_millis(10));

            let event = mouse_state
                .mouse_input(b, WinitElementState::Released)
                .unwrap();
            let expected = MouseState::new_mouse_event(
                [0.0, 0.0],
                None,
                None,
                None,
                Some(MouseInput::Click {
                    click_state: ElementState::Released(1),
                    button: logical_b,
                }),
            );
            assert_eq!(*event.raw_event(), expected);

            // --- Test double click ---
            let _ = mouse_state.mouse_input(b, WinitElementState::Pressed);
            thread::sleep(Duration::from_millis(10));
            let _ = mouse_state.mouse_input(b, WinitElementState::Released);
            thread::sleep(COMBO_DURATION - Duration::from_millis(20)); // within combo duration
            let event = mouse_state
                .mouse_input(b, WinitElementState::Pressed)
                .unwrap();
            let expected = MouseState::new_mouse_event(
                [0.0, 0.0],
                None,
                None,
                None,
                Some(MouseInput::Click {
                    click_state: ElementState::Pressed(2), // Combo count = 2
                    button: logical_b,
                }),
            );
            assert_eq!(*event.raw_event(), expected);
            let _ = mouse_state.mouse_input(b, WinitElementState::Released);

            // --- Test long press ---
            let _ = mouse_state.mouse_input(b, WinitElementState::Pressed);
            thread::sleep(LONG_PRESS_DURATION);

            let events = mouse_state.long_pressing_detection();
            let expected = MouseState::new_mouse_event(
                [0.0, 0.0],
                None,
                None,
                None,
                Some(MouseInput::Click {
                    click_state: ElementState::LongPressed(1), // Combo is reset
                    button: logical_b,
                }),
            );
            assert_eq!(*events[0].raw_event(), expected);
            let _ = mouse_state.mouse_input(b, WinitElementState::Released);
        }
    }

    #[test]
    fn move_and_drag() {
        let mut mouse_state = MouseState::new(
            COMBO_DURATION,
            LONG_PRESS_DURATION,
            MousePrimaryButton::Left,
            PIXEL_PER_LINE,
        )
        .unwrap();

        // --- Test single button dragging ---
        let b = WinitMouseButton::Left;
        let logical_b = mouse_state.to_logical_button(b).unwrap();

        let event = mouse_state.cursor_moved(PhysicalPosition::new(0.0, 0.0));
        let expected_event = MouseState::new_mouse_event([0.0, 0.0], None, None, None, None);
        assert_eq!(*event.raw_event(), expected_event);

        let _ = mouse_state.mouse_input(b, WinitElementState::Pressed);
        thread::sleep(Duration::from_millis(10));

        let event = mouse_state.cursor_moved(PhysicalPosition::new(1.0, 1.0));
        let expected_event = DeviceEventData::MouseEvent {
            current_position: [1.0, 1.0],
            dragging_primary: if logical_b == MouseLogicalButton::Primary {
                Some([0.0, 0.0])
            } else {
                None
            },
            dragging_secondary: if logical_b == MouseLogicalButton::Secondary {
                Some([0.0, 0.0])
            } else {
                None
            },
            dragging_middle: if logical_b == MouseLogicalButton::Middle {
                Some([0.0, 0.0])
            } else {
                None
            },
            event: None,
        };
        assert_eq!(*event.raw_event(), expected_event);

        // Elapse time for long press, but it shouldn't trigger because we are dragging
        thread::sleep(LONG_PRESS_DURATION);

        let events = mouse_state.long_pressing_detection();
        assert_eq!(events.len(), 0);

        let event = mouse_state
            .mouse_input(b, WinitElementState::Released)
            .unwrap();
        let expected_event = MouseState::new_mouse_event(
            [1.0, 1.0], // position is updated
            None,
            None,
            None,
            Some(MouseInput::Click {
                click_state: ElementState::Released(1),
                button: logical_b,
            }),
        );
        assert_eq!(*event.raw_event(), expected_event);
    }
}
