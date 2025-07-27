use std::time::Duration;

use crate::{
    component::Component,
    device::{keyboard_state::KeyboardState, mouse_state::MouseState},
    observer::Observer,
    ui::Widget,
};

const DOUBLE_CLICK_THRESHOLD: Duration = Duration::from_millis(300);
const LONG_PRESS_THRESHOLD: Duration = Duration::from_millis(500);
const SCROLL_PIXEL_PER_LINE: f32 = 40.0;

pub struct UiControl<
    Model: Send + Sync + 'static,
    Message: 'static,
    Response: 'static,
    IR: 'static = Response,
> {
    component: Component<Model, Message, Response, IR>,
    widget: Option<Box<dyn Widget<Response>>>,
    observer: Observer,

    mouse_state: MouseState,
    keyboard_state: KeyboardState,

    scroll_pixel_per_line: f32,
    default_font_size: f32,

    frame_count: u128,
}

impl<Model: Send + Sync + 'static, Message: 'static, Response: 'static, IR: 'static>
    UiControl<Model, Message, Response, IR>
{
    pub fn new(component: Component<Model, Message, Response, IR>) -> Self {
        Self {
            component,
            widget: None,
            observer: Observer::new(),
            mouse_state: MouseState::new(DOUBLE_CLICK_THRESHOLD, LONG_PRESS_THRESHOLD)
                .expect("combo_duration must be less than or equal to long_press_duration"),
            keyboard_state: KeyboardState::new(),
            scroll_pixel_per_line: SCROLL_PIXEL_PER_LINE,
            default_font_size: 16.0,
            frame_count: 0,
        }
    }
}
