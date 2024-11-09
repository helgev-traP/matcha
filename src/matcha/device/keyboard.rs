use bitflags::bitflags;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Key {
    Character(char),
    Spacial(NamedKey),
    Modifiers(Modifiers),
}

impl Key {
    pub fn to_char(&self) -> Option<char> {
        match self {
            Key::Character(c) => Some(*c),
            Key::Spacial(NamedKey::Return) => Some('\n'),
            Key::Spacial(NamedKey::Tab) => Some('\t'),
            Key::Spacial(NamedKey::Space) => Some(' '),
            _ => None,
        }
    }
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub enum KeyLocation {
//     Standard,
//     Left,
//     Right,
//     Numpad,
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum NamedKey {
    // sort by usb hid usage page
    Return,
    Escape,
    Backspace,
    Tab,
    Space,
    // GraveAccent,
    CapsLock,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    PrintScreen,
    ScrollLock,
    Pause,
    Insert,
    Home,
    PageUp,
    Delete,
    End,
    PageDown,
    ArrowRight,
    ArrowLeft,
    ArrowDown,
    ArrowUp,
    NumLock,
}

impl NamedKey {
    pub fn from_winit_named_key(key: winit::keyboard::NamedKey) -> Option<Self> {
        match key {
            winit::keyboard::NamedKey::Enter => Some(NamedKey::Return),
            winit::keyboard::NamedKey::Escape => Some(NamedKey::Escape),
            winit::keyboard::NamedKey::Backspace => Some(NamedKey::Backspace),
            winit::keyboard::NamedKey::Tab => Some(NamedKey::Tab),
            winit::keyboard::NamedKey::Space => Some(NamedKey::Space),
            winit::keyboard::NamedKey::CapsLock => Some(NamedKey::CapsLock),
            winit::keyboard::NamedKey::F1 => Some(NamedKey::F1),
            winit::keyboard::NamedKey::F2 => Some(NamedKey::F2),
            winit::keyboard::NamedKey::F3 => Some(NamedKey::F3),
            winit::keyboard::NamedKey::F4 => Some(NamedKey::F4),
            winit::keyboard::NamedKey::F5 => Some(NamedKey::F5),
            winit::keyboard::NamedKey::F6 => Some(NamedKey::F6),
            winit::keyboard::NamedKey::F7 => Some(NamedKey::F7),
            winit::keyboard::NamedKey::F8 => Some(NamedKey::F8),
            winit::keyboard::NamedKey::F9 => Some(NamedKey::F9),
            winit::keyboard::NamedKey::F10 => Some(NamedKey::F10),
            winit::keyboard::NamedKey::F11 => Some(NamedKey::F11),
            winit::keyboard::NamedKey::F12 => Some(NamedKey::F12),
            winit::keyboard::NamedKey::PrintScreen => Some(NamedKey::PrintScreen),
            winit::keyboard::NamedKey::ScrollLock => Some(NamedKey::ScrollLock),
            winit::keyboard::NamedKey::Pause => Some(NamedKey::Pause),
            winit::keyboard::NamedKey::Insert => Some(NamedKey::Insert),
            winit::keyboard::NamedKey::Home => Some(NamedKey::Home),
            winit::keyboard::NamedKey::PageUp => Some(NamedKey::PageUp),
            winit::keyboard::NamedKey::Delete => Some(NamedKey::Delete),
            winit::keyboard::NamedKey::End => Some(NamedKey::End),
            winit::keyboard::NamedKey::PageDown => Some(NamedKey::PageDown),
            winit::keyboard::NamedKey::ArrowRight => Some(NamedKey::ArrowRight),
            winit::keyboard::NamedKey::ArrowLeft => Some(NamedKey::ArrowLeft),
            winit::keyboard::NamedKey::ArrowDown => Some(NamedKey::ArrowDown),
            winit::keyboard::NamedKey::ArrowUp => Some(NamedKey::ArrowUp),
            winit::keyboard::NamedKey::NumLock => Some(NamedKey::NumLock),
            _ => None,
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Modifiers: u8 {
        const LEFT_SHIFT = 0b0001;
        const RIGHT_SHIFT = 0b0010;
        const LEFT_CTRL = 0b0100;
        const RIGHT_CTRL = 0b1000;
        const LEFT_ALT = 0b0001_0000;
        const RIGHT_ALT = 0b0010_0000;
        const LEFT_SUPER = 0b0100_0000;
        const RIGHT_SUPER = 0b1000_0000;

        const SHIFT = Self::LEFT_SHIFT.bits() | Self::RIGHT_SHIFT.bits();
        const CTRL = Self::LEFT_CTRL.bits() | Self::RIGHT_CTRL.bits();
        const ALT = Self::LEFT_ALT.bits() | Self::RIGHT_ALT.bits();
        const SUPER = Self::LEFT_SUPER.bits() | Self::RIGHT_SUPER.bits();
    }
}

impl Modifiers {
    pub fn from_winit_named_key(
        key: winit::keyboard::NamedKey,
        location: winit::keyboard::KeyLocation,
    ) -> Option<Self> {
        match key {
            winit::keyboard::NamedKey::Control => match location {
                winit::keyboard::KeyLocation::Left => Some(Modifiers::LEFT_CTRL),
                winit::keyboard::KeyLocation::Right => Some(Modifiers::RIGHT_CTRL),
                _ => None,
            },
            winit::keyboard::NamedKey::Alt => match location {
                winit::keyboard::KeyLocation::Left => Some(Modifiers::LEFT_ALT),
                winit::keyboard::KeyLocation::Right => Some(Modifiers::RIGHT_ALT),
                _ => None,
            },
            winit::keyboard::NamedKey::Shift => match location {
                winit::keyboard::KeyLocation::Left => Some(Modifiers::LEFT_SHIFT),
                winit::keyboard::KeyLocation::Right => Some(Modifiers::RIGHT_SHIFT),
                _ => None,
            },
            winit::keyboard::NamedKey::Super => match location {
                winit::keyboard::KeyLocation::Left => Some(Modifiers::LEFT_SUPER),
                winit::keyboard::KeyLocation::Right => Some(Modifiers::RIGHT_SUPER),
                _ => None,
            },
            _ => None,
        }
    }
}
