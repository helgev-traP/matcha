use crate::{
    device::mouse::MouseButton,
    events::{ConcreteEvent, ElementState, Event},
};

// click status: 0 - released, 1 - pressed, 2 - long pressed
pub struct MouseState {
    combo_duration: u64,
    long_press_duration: u64,

    position: [f32; 2],

    primary_click_status: u8,
    primary_dragging_from: Option<[f32; 2]>,
    primary_last_clicked_at: u64,
    primary_click_combo: u32,

    secondary_click_status: u8,
    secondary_dragging_from: Option<[f32; 2]>,
    secondary_last_clicked_at: u64,
    secondary_click_combo: u32,

    middle_click_status: u8,
    middle_dragging_from: Option<[f32; 2]>,
    middle_last_clicked_at: u64,
    middle_click_combo: u32,
}

impl MouseState {
    pub fn new(combo_duration: u64, long_press_duration: u64) -> Option<Self> {
        if combo_duration <= long_press_duration {
            Some(Self {
                combo_duration,
                long_press_duration,
                position: [0.0, 0.0],
                primary_click_status: 0,
                primary_dragging_from: None,
                primary_last_clicked_at: 0,
                primary_click_combo: 0,
                secondary_click_status: 0,
                secondary_dragging_from: None,
                secondary_last_clicked_at: 0,
                secondary_click_combo: 0,
                middle_click_status: 0,
                middle_dragging_from: None,
                middle_last_clicked_at: 0,
                middle_click_combo: 0,
            })
        } else {
            None
        }
    }

    pub fn long_pressing_detection(&mut self, frame: u64) -> Vec<Event> {
        let mut events = Vec::new();
        if self.primary_click_status == 1
            && self.primary_dragging_from.is_none()
            && frame - self.primary_last_clicked_at >= self.long_press_duration
        {
            self.primary_click_status = 2;
            events.push(Event::new(
                frame,
                ConcreteEvent::MouseClick {
                    position: self.position,
                    click_state: ElementState::LongPressed(self.primary_click_combo),
                    button: MouseButton::Primary,
                },
            ));
        }
        if self.secondary_click_status == 1
            && self.secondary_dragging_from.is_none()
            && frame - self.secondary_last_clicked_at >= self.long_press_duration
        {
            self.secondary_click_status = 2;
            events.push(Event::new(
                frame,
                ConcreteEvent::MouseClick {
                    position: self.position,
                    click_state: ElementState::LongPressed(self.secondary_click_combo),
                    button: MouseButton::Secondary,
                },
            ));
        }
        if self.middle_click_status == 1
            && self.middle_dragging_from.is_none()
            && frame - self.middle_last_clicked_at >= self.long_press_duration
        {
            self.middle_click_status = 2;
            events.push(Event::new(
                frame,
                ConcreteEvent::MouseClick {
                    position: self.position,
                    click_state: ElementState::LongPressed(self.middle_click_combo),
                    button: MouseButton::Middle,
                },
            ));
        }
        events
    }

    pub fn button_pressed(&mut self, frame: u64, button: MouseButton) -> Event {
        match button {
            MouseButton::Primary => {
                // update  status
                if frame - self.primary_last_clicked_at <= self.combo_duration {
                    self.primary_click_combo += 1;
                } else {
                    self.primary_click_combo = 1;
                }

                self.primary_last_clicked_at = frame;

                self.primary_click_status = 1;

                Event::new(
                    frame,
                    ConcreteEvent::MouseClick {
                        position: self.position,
                        click_state: ElementState::Pressed(self.primary_click_combo),
                        button: MouseButton::Primary,
                    },
                )
            }
            MouseButton::Secondary => {
                // update  status
                if frame - self.secondary_last_clicked_at <= self.combo_duration {
                    self.secondary_click_combo += 1;
                } else {
                    self.secondary_click_combo = 1;
                }

                self.secondary_last_clicked_at = frame;

                self.secondary_click_status = 1;

                Event::new(
                    frame,
                    ConcreteEvent::MouseClick {
                        position: self.position,
                        click_state: ElementState::Pressed(self.secondary_click_combo),
                        button: MouseButton::Secondary,
                    },
                )
            }
            MouseButton::Middle => {
                // update  status
                if frame - self.middle_last_clicked_at <= self.combo_duration {
                    self.middle_click_combo += 1;
                } else {
                    self.middle_click_combo = 1;
                }

                self.middle_last_clicked_at = frame;

                self.middle_click_status = 1;

                Event::new(
                    frame,
                    ConcreteEvent::MouseClick {
                        position: self.position,
                        click_state: ElementState::Pressed(self.middle_click_combo),
                        button: MouseButton::Middle,
                    },
                )
            }
        }
    }

    pub fn mouse_move(&mut self, frame: u64, position: [f32; 2]) -> Event {
        let primary_dragging_from;
        let secondary_dragging_from;
        let middle_dragging_from;

        if let Some(last_position) = self.primary_dragging_from {
            primary_dragging_from = Some(last_position);
        } else if self.primary_click_status != 0 {
            self.primary_dragging_from = Some(self.position);
            primary_dragging_from = Some(self.position);
        } else {
            primary_dragging_from = None;
        }

        if let Some(last_position) = self.secondary_dragging_from {
            secondary_dragging_from = Some(last_position);
        } else if self.secondary_click_status != 0 {
            self.secondary_dragging_from = Some(self.position);
            secondary_dragging_from = Some(self.position);
        } else {
            secondary_dragging_from = None;
        }

        if let Some(last_position) = self.middle_dragging_from {
            middle_dragging_from = Some(last_position);
        } else if self.middle_click_status != 0 {
            self.middle_dragging_from = Some(self.position);
            middle_dragging_from = Some(self.position);
        } else {
            middle_dragging_from = None;
        }

        self.position = position;

        Event::new(
            frame,
            ConcreteEvent::CursorMove {
                position,
                primary_dragging_from,
                secondary_dragging_from,
                middle_dragging_from,
            },
        )
    }

    pub fn button_released(&mut self, frame: u64, button: MouseButton) -> Event {
        match button {
            MouseButton::Primary => {
                self.primary_click_status = 0;
                self.primary_dragging_from = None;

                Event::new(
                    frame,
                    ConcreteEvent::MouseClick {
                        position: self.position,
                        click_state: ElementState::Released(self.primary_click_combo),
                        button: MouseButton::Primary,
                    },
                )
            }
            MouseButton::Secondary => {
                self.secondary_click_status = 0;
                self.secondary_dragging_from = None;

                Event::new(
                    frame,
                    ConcreteEvent::MouseClick {
                        position: self.position,
                        click_state: ElementState::Released(self.secondary_click_combo),
                        button: MouseButton::Secondary,
                    },
                )
            }
            MouseButton::Middle => {
                self.middle_click_status = 0;
                self.middle_dragging_from = None;

                Event::new(
                    frame,
                    ConcreteEvent::MouseClick {
                        position: self.position,
                        click_state: ElementState::Released(self.middle_click_combo),
                        button: MouseButton::Middle,
                    },
                )
            }
        }
    }

    pub fn cursor_entered(&self, frame: u64) -> Event {
        Event::new(frame, ConcreteEvent::CursorEntered)
    }

    pub fn cursor_left(&self, frame: u64) -> Event {
        Event::new(frame, ConcreteEvent::CursorLeft)
    }

    pub fn mouse_scroll(&self, frame: u64, delta: [f32; 2]) -> Event {
        Event::new(
            frame,
            ConcreteEvent::MouseScroll {
                position: self.position,
                delta,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn click_and_long_press() {
        let mut mouse_state = MouseState::new(10, 100).unwrap();
        let mut frame = 0;
        for b in [
            MouseButton::Primary,
            MouseButton::Secondary,
            MouseButton::Middle,
        ] {
            for i in 0..10 {
                let event = mouse_state.button_pressed(frame, b);
                assert_eq!(
                    event.raw_event(),
                    ConcreteEvent::MouseClick {
                        position: [0.0, 0.0],
                        click_state: ElementState::Pressed(i + 1),
                        button: b,
                    }
                );
                frame += 1;

                let event = mouse_state.button_released(frame, b);
                assert_eq!(
                    event.raw_event(),
                    ConcreteEvent::MouseClick {
                        position: [0.0, 0.0],
                        click_state: ElementState::Released(i + 1),
                        button: b,
                    }
                );
                frame += 1;
            }

            let _ = mouse_state.button_pressed(frame, b);

            frame += 100;

            let event = mouse_state.long_pressing_detection(frame);
            assert_eq!(
                event[0].raw_event(),
                ConcreteEvent::MouseClick {
                    position: [0.0, 0.0],
                    click_state: ElementState::LongPressed(11),
                    button: b,
                }
            );

            frame += 1;

            let event = mouse_state.button_released(frame, b);

            assert_eq!(
                event.raw_event(),
                ConcreteEvent::MouseClick {
                    position: [0.0, 0.0],
                    click_state: ElementState::Released(11),
                    button: b,
                }
            );
        }
    }

    #[test]
    fn move_and_drag() {
        let mut mouse_state = MouseState::new(10, 100).unwrap();
        let mut frame = 0;

        // single button pressing
        for b in [
            MouseButton::Primary,
            MouseButton::Secondary,
            MouseButton::Middle,
        ] {
            let event = mouse_state.mouse_move(frame, [0.0, 0.0]);
            assert_eq!(
                event.raw_event(),
                ConcreteEvent::CursorMove {
                    position: [0.0, 0.0],
                    primary_dragging_from: None,
                    secondary_dragging_from: None,
                    middle_dragging_from: None,
                }
            );
            frame += 1;

            let _ = mouse_state.button_pressed(frame, b);
            frame += 1;

            let event = mouse_state.mouse_move(frame, [1.0, 1.0]);
            assert_eq!(
                event.raw_event(),
                ConcreteEvent::CursorMove {
                    position: [1.0, 1.0],
                    primary_dragging_from: if b == MouseButton::Primary {
                        Some([0.0, 0.0])
                    } else {
                        None
                    },
                    secondary_dragging_from: if b == MouseButton::Secondary {
                        Some([0.0, 0.0])
                    } else {
                        None
                    },
                    middle_dragging_from: if b == MouseButton::Middle {
                        Some([0.0, 0.0])
                    } else {
                        None
                    },
                }
            );
            frame += 100;

            let event = mouse_state.long_pressing_detection(frame);
            assert_eq!(event.len(), 0,);
            frame += 1;

            let event = mouse_state.button_released(frame, b);
            assert_eq!(
                event.raw_event(),
                ConcreteEvent::MouseClick {
                    position: [1.0, 1.0],
                    click_state: ElementState::Released(1),
                    button: b,
                }
            );
            frame += 1;
        }

        // multiple button pressing
        {
            let _ = mouse_state.mouse_move(frame, [0.0, 0.0]);
            frame += 1;
            let _ = mouse_state.button_pressed(frame, MouseButton::Primary);
            frame += 1;

            let event = mouse_state.mouse_move(frame, [1.0, 1.0]);
            assert_eq!(
                event.raw_event(),
                ConcreteEvent::CursorMove {
                    position: [1.0, 1.0],
                    primary_dragging_from: Some([0.0, 0.0]),
                    secondary_dragging_from: None,
                    middle_dragging_from: None,
                }
            );
            frame += 1;

            let _ = mouse_state.button_pressed(frame, MouseButton::Secondary);
            frame += 1;

            let event = mouse_state.mouse_move(frame, [2.0, 2.0]);
            assert_eq!(
                event.raw_event(),
                ConcreteEvent::CursorMove {
                    position: [2.0, 2.0],
                    primary_dragging_from: Some([0.0, 0.0]),
                    secondary_dragging_from: Some([1.0, 1.0]),
                    middle_dragging_from: None,
                }
            );
            frame += 1;

            let _ = mouse_state.button_pressed(frame, MouseButton::Middle);
            frame += 1;

            let event = mouse_state.mouse_move(frame, [3.0, 3.0]);
            assert_eq!(
                event.raw_event(),
                ConcreteEvent::CursorMove {
                    position: [3.0, 3.0],
                    primary_dragging_from: Some([0.0, 0.0]),
                    secondary_dragging_from: Some([1.0, 1.0]),
                    middle_dragging_from: Some([2.0, 2.0]),
                }
            );
        }
    }
}
