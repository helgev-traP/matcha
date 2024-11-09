use super::{component::Component, types::color::Color, window::Window};

pub struct App<'a, Model: Send + 'static, Message: 'static> {
    window: Window<'a, Model, Message>,
}

impl<Model: Send + 'static, Message: 'static> App<'_, Model, Message> {
    pub fn new(component: Component<Model, Message, Message, Message>) -> Self {
        Self {
            window: Window::new(component),
        }
    }

    pub fn title(mut self, title: &str) -> Self {
        self.window.title(title);
        self
    }

    pub fn base_color(mut self, color: Color) -> Self {
        self.window.base_color(color);
        self
    }

    pub fn run(&mut self) {
        let event_loop = winit::event_loop::EventLoop::with_user_event().build().unwrap();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        let _ = event_loop.run_app(&mut self.window);
    }
}
