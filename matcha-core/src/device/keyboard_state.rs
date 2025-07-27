use crate::events::{ConcreteEvent, Event, KeyEvent};
use std::collections::{HashMap, VecDeque};
use winit::keyboard::{Key as WinitKey, KeyCode, ModifiersState, PhysicalKey};

/// キーボードの特定時点での状態のスナップショット。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct KeyboardSnapshot {
    /// 現在押されている物理キーと、対応する論理キーのマッピング。
    pressed: HashMap<KeyCode, WinitKey>,
    /// キーが押された順番を保持するキュー（最新が末尾）。
    press_order: VecDeque<KeyCode>,
    /// 現在の修飾キーの状態。
    modifiers: ModifiersState,
}

impl KeyboardSnapshot {
    /// 指定した物理キーが現在押されているかを確認します。
    pub fn is_physical_pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains_key(&key)
    }

    /// 指定した論理キーが現在押されているかを確認します。
    pub fn is_logical_pressed(&self, key: &WinitKey) -> bool {
        self.pressed.values().any(|v| v == key)
    }

    /// 現在の修飾キーの状態を取得します。
    pub fn modifiers(&self) -> ModifiersState {
        self.modifiers
    }

    /// 押されているキーを押された順にイテレータとして取得します。
    pub fn press_order(&self) -> impl Iterator<Item = (&KeyCode, &WinitKey)> {
        self.press_order.iter().rev().map(|k| (k, &self.pressed[k]))
    }
}

/// winitのキーイベントを処理し、KeyboardSnapshotを管理する状態マシン。
#[derive(Default)]
pub struct KeyboardState {
    snapshot: KeyboardSnapshot,
}

impl KeyboardState {
    pub fn new() -> Self {
        Self::default()
    }

    /// winitのイベントを受け取り、内部状態を更新し、アプリケーションのKeyEventを生成する。
    pub fn process_winit_event(
        &mut self,
        winit_event: &winit::event::KeyEvent,
        modifiers: ModifiersState,
    ) -> Event {
        // 1. `self.snapshot`を更新する
        if let PhysicalKey::Code(key_code) = winit_event.physical_key {
            match winit_event.state {
                winit::event::ElementState::Pressed => {
                    if !self.snapshot.pressed.contains_key(&key_code) {
                        self.snapshot
                            .pressed
                            .insert(key_code, winit_event.logical_key.clone());
                        self.snapshot.press_order.push_back(key_code);
                    }
                }
                winit::event::ElementState::Released => {
                    if self.snapshot.pressed.remove(&key_code).is_some() {
                        self.snapshot.press_order.retain(|&k| k != key_code);
                    }
                }
            }
        }

        // winitから最新の修飾キー状態を受け取り、そのままスナップショットに反映
        self.snapshot.modifiers = modifiers;

        // 2. アプリケーションのKeyEventを生成して返す
        let key_event = KeyEvent {
            winit: winit_event.clone(),
            snapshot: self.snapshot.clone(), // 更新された最新の状態のスナップショットを添付
        };

        Event::new(ConcreteEvent::KeyboardEvent(key_event))
    }

    /// 現在のキーボード状態への読み取り専用アクセスを提供する。
    pub fn snapshot(&self) -> &KeyboardSnapshot {
        &self.snapshot
    }
}
