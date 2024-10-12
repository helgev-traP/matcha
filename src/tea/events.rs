// widget event

use std::time::Instant;

pub struct UiEvent {
    pub time: Instant,
    pub content: UiEventContent,
}

pub enum UiEventContent {
    None,
    // mouse event
    CursorMoved {
        x: f32,
        y: f32,
    },
    MousePressed {
        x: f32,
        y: f32,
        button: MouseButton,
        combo: u32,
    },
    MouseReleased {
        x: f32,
        y: f32,
        button: MouseButton,
    },
    MouseWheel {
        x: f32,
        y: f32,
        delta: [f32; 2],
    },
    Drag {
        x: f32,
        y: f32,
        from_x: f32,
        from_y: f32,
        button: MouseButton,
    },
    // keyboard event
    // todo
}

pub struct UiEventResult<UserEvent> {
    // matcha-ui system event

    // pub system_event: Option<{SystemMessage}>,

    // user event
    pub user_event: Option<UserEvent>,
}

impl<UserEvent> UiEventResult<UserEvent> {
    pub fn swap_user_event<UserEvent2>(self, event: UserEvent2) -> UiEventResult<UserEvent2> {
        UiEventResult {
            user_event: Some(event),
        }
    }
}

impl<R> Default for UiEventResult<R> {
    fn default() -> Self {
        UiEventResult { user_event: None }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Primary,
    Secondary,
    Middle,
}