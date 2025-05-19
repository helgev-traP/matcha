// widget event
use super::device::{keyboard::Key, mouse::MouseButton};

// MARK: Event

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Event {
    frame: u64,
    raw: ConcreteEvent,
    relative_position: [f32; 2],
}

impl Event {
    pub fn new(frame: u64, event: ConcreteEvent) -> Self {
        Self {
            frame,
            raw: event,
            relative_position: [0.0, 0.0],
        }
    }

    pub fn frame(&self) -> u64 {
        self.frame
    }

    pub fn raw_event(&self) -> ConcreteEvent {
        self.raw
    }

    pub fn event(&self) -> ConcreteEvent {
        todo!()
    }

    pub fn transition(&self, position: [f32; 2]) -> Self {
        Self {
            frame: self.frame,
            raw: self.raw,
            relative_position: [
                self.relative_position[0] + position[0],
                self.relative_position[1] + position[1],
            ],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ConcreteEvent {
    #[default]
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

// MARK: ElementState

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
