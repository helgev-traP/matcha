use super::{ElementState, KeyboardState};
use winit::{event::KeyEvent as RawKeyEvent, keyboard::NamedKey};

pub use winit::keyboard::{Key, KeyCode, KeyLocation, ModifiersState, PhysicalKey};

/// A keyboard event.
///
/// This struct contains the raw `winit::event::KeyEvent` and a snapshot of the
/// entire keyboard state at the moment the event occurred.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyInput {
    pub winit: RawKeyEvent,
    pub snapshot: KeyboardState,
}

// --- Methods for the key that triggered the event ---

impl KeyInput {
    /// Returns `true` if the logical key that triggered this event is `CapsLock`.
    pub fn caps_lock(&self) -> bool {
        if let Key::Named(named_key) = self.logical_key() {
            *named_key == NamedKey::CapsLock
        } else {
            false
        }
    }

    // todo: Implement a rest of keys
}

/// Raw information about the key that triggered this event.
///
/// These methods delegate directly to the underlying `winit::event::KeyEvent`.
impl KeyInput {
    pub fn physical_key(&self) -> PhysicalKey {
        self.winit.physical_key
    }

    pub fn logical_key(&self) -> &Key {
        &self.winit.logical_key
    }

    pub fn text(&self) -> Option<&str> {
        self.winit.text.as_deref()
    }

    pub fn location(&self) -> KeyLocation {
        self.winit.location
    }

    pub fn state(&self) -> ElementState {
        self.winit.state.into()
    }

    pub fn is_repeat(&self) -> bool {
        self.winit.repeat
    }
}

// --- Methods for the keyboard state at the moment of the event ---

/// Methods to query the state of modifier keys at the moment the event occurred.
impl KeyInput {
    /// Returns `true` if the `Control` key was held down when the event occurred.
    pub fn ctrl_held(&self) -> bool {
        self.modifiers().control_key()
    }

    /// Returns `true` if the `Shift` key was held down when the event occurred.
    pub fn shift_held(&self) -> bool {
        self.modifiers().shift_key()
    }

    /// Returns `true` if the `Alt` key was held down when the event occurred.
    pub fn alt_held(&self) -> bool {
        self.modifiers().alt_key()
    }

    /// Returns `true` if the `Super` key (e.g., Windows or Command) was held down when the event occurred.
    pub fn super_held(&self) -> bool {
        self.modifiers().super_key()
    }
}

/// General information about the keyboard state at the moment the event occurred.
///
/// These methods query the `KeyboardState` snapshot and are not limited to
/// the key that triggered the event.
impl KeyInput {
    /// Returns `true` if the given physical key was held down when the event occurred.
    pub fn is_physical_pressed(&self, key: KeyCode) -> bool {
        self.snapshot.is_physical_pressed(&key)
    }

    /// Returns `true` if the given logical key was held down when the event occurred.
    pub fn is_logical_pressed(&self, key: Key) -> bool {
        self.snapshot.is_logical_pressed(&key)
    }

    /// Returns the state of the modifier keys at the moment the event occurred.
    pub fn modifiers(&self) -> ModifiersState {
        self.snapshot.modifiers()
    }

    /// Returns a list of all keys currently held down, in the order they were pressed.
    pub fn press_order(&self) -> Vec<(KeyCode, Key)> {
        self.snapshot.press_order()
    }
}
