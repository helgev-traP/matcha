// widget event
use super::device::{
    keyboard::Key,
    mouse::MouseButton,
};

#[derive(Debug, Clone, PartialEq)]
pub struct UiEvent {
    pub frame: u64,
    pub content: UiEventContent,
    pub diff: (), // todo
}

#[derive(Debug, Clone, PartialEq)]
pub enum UiEventContent {
    None,
    // mouse event
    MouseClick {
        position: [f32; 2],
        click_state: ElementState,
        button: MouseButton,
    },
    CursorMove {
        position: [f32; 2],
        primary_dragging_from: Option<[f32; 2]>,
        secondary_dragging_from: Option<[f32; 2]>,
        middle_dragging_from: Option<[f32; 2]>,
    },
    CursorEntered,
    CursorLeft,
    MouseScroll {
        position: [f32; 2],
        delta: [f32; 2],
    },
    // keyboard event
    KeyboardInput {
        key: Key,
        element_state: ElementState,
    },
    // todo
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElementState {
    Pressed(u32),
    LongPressed(u32),
    Released(u32),
}

impl ElementState {
    pub(crate) fn from_winit_state(state: winit::event::ElementState, count: u32) -> Self {
        match state {
            winit::event::ElementState::Pressed => ElementState::Pressed(count),
            winit::event::ElementState::Released => ElementState::Released(count),
        }
    }
}

// result of widget event

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
