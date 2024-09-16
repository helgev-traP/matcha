use super::window::Window;

pub struct App<'a> {
    // model
    // view
    window: Window<'a>,
    // update
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        Self {
            window: Window::new(),
        }
    }

    pub fn run(&mut self, title: &str) {
        self.window.set_title(title);
        let event_loop = winit::event_loop::EventLoop::new().unwrap();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
        event_loop.run_app(&mut self.window).unwrap();
    }

    pub fn run_with_cosmic_text(&mut self, title: &str, cosmic_text: crate::cosmic::FontContext) {
        self.window.set_title(title);
        let event_loop = winit::event_loop::EventLoop::new().unwrap();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
        event_loop.run_app(&mut self.window).unwrap();
    }
}
