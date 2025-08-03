pub mod button_state;
pub mod element_state;
pub mod key_input;
pub mod key_state;
pub mod mouse_input;
pub mod mouse_state;
pub mod window_state;

use std::path::PathBuf;

use button_state::ButtonState;
pub use element_state::ElementState;
pub use key_input::{Key, KeyCode, KeyInput, KeyLocation, ModifiersState, PhysicalKey};
pub use key_state::KeyboardState;
pub use mouse_input::MouseInput;
pub use mouse_input::MouseLogicalButton;
pub use mouse_state::MouseState;
pub use winit::window::Theme;

// MARK: Event

/// Represents a generic UI event within the application.
#[derive(Debug, Clone, PartialEq)]
pub struct DeviceEvent {
    /// raw event.
    raw: DeviceEventData,
    /// relative event.
    relative: DeviceEventData,
}

impl DeviceEvent {
    /// Creates a new `Event` from a `ConcreteEvent`.
    pub(crate) fn new(event: DeviceEventData) -> Self {
        Self {
            raw: event.clone(),
            relative: event,
        }
    }

    /// Returns a reference to the raw concrete event.
    pub fn raw_event(&self) -> &DeviceEventData {
        &self.raw
    }

    /// Placeholder for a more advanced event processing method.
    pub fn event(&self) -> &DeviceEventData {
        &self.relative
    }

    pub fn relative(&self, event: DeviceEventData) -> Self {
        Self {
            raw: self.raw.clone(),
            relative: event,
        }
    }

    /// Creates a new `Event` with an updated relative position.
    /// Used for propagating events down the widget tree.
    pub fn mouse_transition(&self, delta: [f32; 2]) -> Self {
        Self {
            raw: self.raw.clone(),
            relative: self.relative.mouse_transition(delta),
        }
    }
}

/// Represents the concrete type of a UI event.
#[derive(Debug, Clone, PartialEq)]
pub enum DeviceEventData {
    WindowPositionSize {
        inner_position: [f32; 2],
        outer_position: [f32; 2],
        inner_size: [f32; 2],
        outer_size: [f32; 2],
    },
    CloseRequested,
    FileDrop {
        mouse_position: [f32; 2],
        path_buf: PathBuf,
    },
    FileHover {
        mouse_position: [f32; 2],
        path_buf: PathBuf,
    },
    FileHoverCancelled {
        mouse_position: [f32; 2],
    },
    WindowFocus(bool),
    Keyboard(KeyInput),
    /// not implemented yet
    Ime,
    MouseEvent {
        current_position: [f32; 2],
        dragging_primary: Option<[f32; 2]>,
        dragging_secondary: Option<[f32; 2]>,
        dragging_middle: Option<[f32; 2]>,
        event: Option<MouseInput>,
    },
    /// not implemented yet
    Touch,
    Theme(Theme),
}

impl DeviceEventData {
    pub fn mouse_transition(&self, delta: [f32; 2]) -> Self {
        match self {
            DeviceEventData::FileDrop {
                mouse_position,
                path_buf,
            } => DeviceEventData::FileDrop {
                mouse_position: [mouse_position[0] - delta[0], mouse_position[1] - delta[1]],
                path_buf: path_buf.clone(),
            },
            DeviceEventData::FileHover {
                mouse_position,
                path_buf,
            } => DeviceEventData::FileHover {
                mouse_position: [mouse_position[0] - delta[0], mouse_position[1] - delta[1]],
                path_buf: path_buf.clone(),
            },
            DeviceEventData::FileHoverCancelled { mouse_position } => {
                DeviceEventData::FileHoverCancelled {
                    mouse_position: [mouse_position[0] - delta[0], mouse_position[1] - delta[1]],
                }
            }
            DeviceEventData::MouseEvent {
                current_position,
                dragging_primary,
                dragging_secondary,
                dragging_middle,
                event,
            } => DeviceEventData::MouseEvent {
                current_position: [
                    current_position[0] - delta[0],
                    current_position[1] - delta[1],
                ],
                dragging_primary: dragging_primary.map(|p| [p[0] - delta[0], p[1] - delta[1]]),
                dragging_secondary: dragging_secondary.map(|p| [p[0] - delta[0], p[1] - delta[1]]),
                dragging_middle: dragging_middle.map(|p| [p[0] - delta[0], p[1] - delta[1]]),
                event: *event,
            },
            // For other events that do not have a mouse position, we clone them.
            DeviceEventData::WindowPositionSize { .. }
            | DeviceEventData::CloseRequested
            | DeviceEventData::WindowFocus(_)
            | DeviceEventData::Keyboard(_)
            | DeviceEventData::Ime
            | DeviceEventData::Touch
            | DeviceEventData::Theme(_) => self.clone(),
        }
    }
}
