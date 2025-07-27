// widget event
use super::device::{keyboard::Key, mouse::MouseButton};

// MARK: Event

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Event {
    raw: ConcreteEvent,
    relative_position: [f32; 2],
}

impl Event {
    pub fn new(event: ConcreteEvent) -> Self {
        Self {
            raw: event,
            relative_position: [0.0, 0.0],
        }
    }

    pub fn raw_event(&self) -> &ConcreteEvent {
        &self.raw
    }

    pub fn event(&self) -> ConcreteEvent {
        todo!()
    }

    pub fn transition(&self, position: [f32; 2]) -> Self {
        Self {
            raw: self.raw.clone(),
            relative_position: [
                self.relative_position[0] + position[0],
                self.relative_position[1] + position[1],
            ],
        }
    }
}

use crate::device::keyboard_state::KeyboardSnapshot;
use winit::event::KeyEvent as WinitKeyEvent;

#[derive(Debug, Clone, PartialEq)]
pub struct KeyEvent {
    /// イベントを発生させたwinitの元イベント。`repeat`, `text`など豊富な情報を持つ。
    pub winit: WinitKeyEvent,
    /// イベント発生瞬間のキーボード全体の状態。
    pub snapshot: KeyboardSnapshot,
}

impl KeyEvent {
    /* --- UIウィジェット向けAPI --- */

    // --- トリガー情報 ---
    /// このイベントを発生させた物理キーを取得します。
    pub fn physical_key(&self) -> winit::keyboard::PhysicalKey {
        self.winit.physical_key
    }
    /// このイベントを発生させた論理キーを取得します。
    pub fn logical_key(&self) -> &winit::keyboard::Key {
        &self.winit.logical_key
    }
    /// このイベントがキーリピートによるものか否かを取得します。
    pub fn is_repeat(&self) -> bool {
        self.winit.repeat
    }
    /// このキー入力によって生成されたテキストを取得します（例: 'a', 'A', '\n'）。
    pub fn text(&self) -> Option<&str> {
        self.winit.text.as_deref()
    }

    // --- 状態スナップショットへのアクセス ---
    /// 指定した物理キーが**イベント発生時に**押されていたかを確認します。
    pub fn is_physical_pressed(&self, key: winit::keyboard::KeyCode) -> bool {
        self.snapshot.is_physical_pressed(key)
    }
    /// 指定した論理キーが**イベント発生時に**押されていたかを確認します。
    pub fn is_logical_pressed(&self, key: &winit::keyboard::Key) -> bool {
        self.snapshot.is_logical_pressed(key)
    }
    /// **イベント発生時**の修飾キーの状態を取得します。
    pub fn modifiers(&self) -> winit::keyboard::ModifiersState {
        self.snapshot.modifiers()
    }
    /// **イベント発生時**に押されていたキーを押された順にイテレータとして取得します。
    pub fn press_order(&self) -> impl Iterator<Item = (&winit::keyboard::KeyCode, &winit::keyboard::Key)> {
        self.snapshot.press_order()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConcreteEvent {
    None,
    // mouse event
    MouseEvent {
        current_position: [f32; 2],
        dragging_primary: Option<[f32; 2]>,
        dragging_secondary: Option<[f32; 2]>,
        dragging_middle: Option<[f32; 2]>,

        event: MouseEvent,
    },
    // keyboard event
    KeyboardEvent(KeyEvent),
    // todo
}

impl Default for ConcreteEvent {
    fn default() -> Self {
        ConcreteEvent::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseEvent {
    Click {
        click_state: ElementState,
        button: MouseButton,
    },
    Move,
    Entered,
    Left,
    Scroll {
        delta: [f32; 2],
    },
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
