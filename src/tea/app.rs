use super::{component::Component, window::Window};

pub struct App<'a, Model, Message: 'static> {
    window: Window<'a, Model, Message>,
}

impl<Model, Message: 'static> App<'_, Model, Message> {
    pub fn new(component: Component<Model, Message, Message>) -> Self {
        Self {
            window: Window::new(component),
        }
    }

    pub fn run(&mut self) {
        let event_loop = winit::event_loop::EventLoop::with_user_event().build().unwrap();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        let _ = event_loop.run_app(&mut self.window);
    }
}
