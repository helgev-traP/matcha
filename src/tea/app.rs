use super::{component::Component, window::Window};

pub struct AppState<'a, Model, Message: 'static> {
    window: Window<'a, Model, Message>,
}

impl<Model, Message: 'static> AppState<'_, Model, Message> {
    pub fn new(component: Component<Model, Message>) -> Self {
        Self {
            window: Window::new(component),
        }
    }

    pub fn run(&mut self) {
        let event_loop = winit::event_loop::EventLoop::with_user_event().build().unwrap();
        let _ = event_loop.run_app(&mut self.window);
    }
}
