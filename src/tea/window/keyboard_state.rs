use std::collections::{HashMap, HashSet};

use winit::keyboard::SmolStr;

use crate::{
    device::keyboard::{Key, Modifiers, NamedKey},
    events::{ElementState, UiEvent, UiEventContent},
};

// track keyboard state by winit structure.
pub struct KeyboardState {
    characters: HashMap<winit::keyboard::KeyCode, (SmolStr, u32)>,
    named_keys: HashMap<NamedKey, u32>,
    modifiers: HashMap<Modifiers, u32>,
}

impl KeyboardState {
    pub fn new() -> Self {
        Self {
            characters: HashMap::new(),
            named_keys: HashMap::new(),
            modifiers: HashMap::new(),
        }
    }

    pub fn key_event(&mut self, frame: u64, event: winit::event::KeyEvent) -> Option<UiEvent> {
        match event.state {
            winit::event::ElementState::Pressed => match event.logical_key {
                winit::keyboard::Key::Named(named_key) => match named_key {
                    winit::keyboard::NamedKey::Enter
                    | winit::keyboard::NamedKey::Escape
                    | winit::keyboard::NamedKey::Backspace
                    | winit::keyboard::NamedKey::Tab
                    | winit::keyboard::NamedKey::Space
                    | winit::keyboard::NamedKey::CapsLock
                    | winit::keyboard::NamedKey::F1
                    | winit::keyboard::NamedKey::F2
                    | winit::keyboard::NamedKey::F3
                    | winit::keyboard::NamedKey::F4
                    | winit::keyboard::NamedKey::F5
                    | winit::keyboard::NamedKey::F6
                    | winit::keyboard::NamedKey::F7
                    | winit::keyboard::NamedKey::F8
                    | winit::keyboard::NamedKey::F9
                    | winit::keyboard::NamedKey::F10
                    | winit::keyboard::NamedKey::F11
                    | winit::keyboard::NamedKey::F12
                    | winit::keyboard::NamedKey::PrintScreen
                    | winit::keyboard::NamedKey::ScrollLock
                    | winit::keyboard::NamedKey::Pause
                    | winit::keyboard::NamedKey::Insert
                    | winit::keyboard::NamedKey::Home
                    | winit::keyboard::NamedKey::PageUp
                    | winit::keyboard::NamedKey::Delete
                    | winit::keyboard::NamedKey::End
                    | winit::keyboard::NamedKey::PageDown
                    | winit::keyboard::NamedKey::ArrowRight
                    | winit::keyboard::NamedKey::ArrowLeft
                    | winit::keyboard::NamedKey::ArrowDown
                    | winit::keyboard::NamedKey::ArrowUp
                    | winit::keyboard::NamedKey::NumLock => {
                        if let Some(count) = self
                            .named_keys
                            .get_mut(&NamedKey::from_winit_named_key(named_key).unwrap())
                        {
                            *count += 1;
                        } else {
                            self.named_keys
                                .insert(NamedKey::from_winit_named_key(named_key).unwrap(), 1);
                        }

                        Some(UiEvent {
                            frame: frame,
                            content: UiEventContent::KeyboardInput {
                                key: Key::Spacial(
                                    NamedKey::from_winit_named_key(named_key).unwrap(),
                                ),
                                element_state: ElementState::from_winit_state(
                                    event.state,
                                    self.named_keys
                                        [&NamedKey::from_winit_named_key(named_key).unwrap()],
                                ),
                            },
                            diff: (),
                        })
                    }
                    winit::keyboard::NamedKey::Control
                    | winit::keyboard::NamedKey::Alt
                    | winit::keyboard::NamedKey::Shift
                    | winit::keyboard::NamedKey::Super => {
                        if let Some(count) = self.modifiers.get_mut(
                            &Modifiers::from_winit_named_key(named_key, event.location).unwrap(),
                        ) {
                            *count += 1;
                        } else {
                            self.modifiers.insert(
                                Modifiers::from_winit_named_key(named_key, event.location).unwrap(),
                                1,
                            );
                        }

                        Some(UiEvent {
                            frame: frame,
                            content: UiEventContent::KeyboardInput {
                                key: Key::Modifiers(
                                    Modifiers::from_winit_named_key(named_key, event.location)
                                        .unwrap(),
                                ),
                                element_state: ElementState::from_winit_state(
                                    event.state,
                                    self.modifiers[&Modifiers::from_winit_named_key(
                                        named_key,
                                        event.location,
                                    )
                                    .unwrap()],
                                ),
                            },
                            diff: (),
                        })
                    }
                    _ => None,
                },
                winit::keyboard::Key::Character(c) => {
                    if let winit::keyboard::PhysicalKey::Code(code) = event.physical_key {
                        if let Some((_, count)) = self.characters.get_mut(&code) {
                            *count += 1;
                        } else {
                            self.characters.insert(code, (c, 1));
                        }

                        Some(UiEvent {
                            frame: frame,
                            content: UiEventContent::KeyboardInput {
                                key: Key::Character(self.characters[&code].0.chars().next().unwrap()),
                                element_state: ElementState::from_winit_state(
                                    event.state,
                                    self.characters[&code].1,
                                ),
                            },
                            diff: (),
                        })
                    } else {
                        None
                    }
                }
                winit::keyboard::Key::Unidentified(_) => None,
                winit::keyboard::Key::Dead(_) => None,
            },

            winit::event::ElementState::Released => match event.logical_key {
                winit::keyboard::Key::Named(named_key) => match named_key {
                    winit::keyboard::NamedKey::Enter
                    | winit::keyboard::NamedKey::Escape
                    | winit::keyboard::NamedKey::Backspace
                    | winit::keyboard::NamedKey::Tab
                    | winit::keyboard::NamedKey::Space
                    | winit::keyboard::NamedKey::CapsLock
                    | winit::keyboard::NamedKey::F1
                    | winit::keyboard::NamedKey::F2
                    | winit::keyboard::NamedKey::F3
                    | winit::keyboard::NamedKey::F4
                    | winit::keyboard::NamedKey::F5
                    | winit::keyboard::NamedKey::F6
                    | winit::keyboard::NamedKey::F7
                    | winit::keyboard::NamedKey::F8
                    | winit::keyboard::NamedKey::F9
                    | winit::keyboard::NamedKey::F10
                    | winit::keyboard::NamedKey::F11
                    | winit::keyboard::NamedKey::F12
                    | winit::keyboard::NamedKey::PrintScreen
                    | winit::keyboard::NamedKey::ScrollLock
                    | winit::keyboard::NamedKey::Pause
                    | winit::keyboard::NamedKey::Insert
                    | winit::keyboard::NamedKey::Home
                    | winit::keyboard::NamedKey::PageUp
                    | winit::keyboard::NamedKey::Delete
                    | winit::keyboard::NamedKey::End
                    | winit::keyboard::NamedKey::PageDown
                    | winit::keyboard::NamedKey::ArrowRight
                    | winit::keyboard::NamedKey::ArrowLeft
                    | winit::keyboard::NamedKey::ArrowDown
                    | winit::keyboard::NamedKey::ArrowUp
                    | winit::keyboard::NamedKey::NumLock => {
                        let count = self
                            .named_keys
                            .get_mut(&NamedKey::from_winit_named_key(named_key).unwrap())
                            .map(|count| *count)
                            .unwrap_or(0);

                        self.named_keys
                            .remove(&NamedKey::from_winit_named_key(named_key).unwrap());

                        Some(UiEvent {
                            frame: frame,
                            content: UiEventContent::KeyboardInput {
                                key: Key::Spacial(
                                    NamedKey::from_winit_named_key(named_key).unwrap(),
                                ),
                                element_state: ElementState::from_winit_state(event.state, count),
                            },
                            diff: (),
                        })
                    }
                    winit::keyboard::NamedKey::Control
                    | winit::keyboard::NamedKey::Alt
                    | winit::keyboard::NamedKey::Shift
                    | winit::keyboard::NamedKey::Super => {
                        let count = self
                            .modifiers
                            .get_mut(
                                &Modifiers::from_winit_named_key(named_key, event.location)
                                    .unwrap(),
                            )
                            .map(|count| *count)
                            .unwrap_or(0);

                        self.modifiers.remove(
                            &Modifiers::from_winit_named_key(named_key, event.location).unwrap(),
                        );

                        Some(UiEvent {
                            frame: frame,
                            content: UiEventContent::KeyboardInput {
                                key: Key::Modifiers(
                                    Modifiers::from_winit_named_key(named_key, event.location)
                                        .unwrap(),
                                ),
                                element_state: ElementState::from_winit_state(event.state, count),
                            },
                            diff: (),
                        })
                    }
                    _ => None,
                },
                winit::keyboard::Key::Character(c) => {
                    if let winit::keyboard::PhysicalKey::Code(code) = event.physical_key {
                        let count = self
                            .characters
                            .get_mut(&code)
                            .map(|(_, count)| *count)
                            .unwrap_or(0);

                        self.characters.remove(&code);

                        Some(UiEvent {
                            frame: frame,
                            content: UiEventContent::KeyboardInput {
                                key: Key::Character(c.chars().next().unwrap()),
                                element_state: ElementState::from_winit_state(event.state, count),
                            },
                            diff: (),
                        })
                    } else {
                        None
                    }
                }
                winit::keyboard::Key::Unidentified(_) => None,
                winit::keyboard::Key::Dead(_) => None,
            },
        }
    }
}
