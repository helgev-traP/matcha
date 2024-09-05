use super::{widgets::Elements, window::Window};

pub struct App<'a> {
    // model
    // view
    window: Window<'a>,
    // update
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        Self { window: Window::new() }
    }

    pub fn set_ui_tree(&mut self, ui_tree: Box<dyn Elements>) {
        self.window.set_ui_tree(ui_tree);
    }

    pub fn set_background_color(&mut self, color: [f64; 4]) {
        self.window.set_background_color(color);
    }

    pub fn run(&mut self) {
        let event_loop = winit::event_loop::EventLoop::new().unwrap();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        event_loop.run_app(&mut self.window).unwrap();
    }
}