use super::{component::Component, types::color::Color, window::Window};

pub struct App<'a, Model: Send + 'static, Message: 'static> {
    window: Window<'a, Model, Message>,
}

impl<Model: Send + 'static, Message: Send + 'static> App<'_, Model, Message> {
    pub fn new(component: Component<Model, Message, Message, Message>) -> Self {
        // todo

        // make runtime
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(10)
            .enable_all()
            .build()
            .unwrap();

        Self {
            window: Window::new(component, rt),
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
        let event_loop = winit::event_loop::EventLoop::with_user_event()
            .build()
            .unwrap();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        let _ = event_loop.run_app(&mut self.window);
    }
}
