mod window;
pub use window::Window;
pub use window::DeviceQueue;
pub mod types;
pub mod widgets;
pub use widgets::Widget;

pub struct App<'a> {
    window: Window<'a>,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        Self { window: Window::new() }
    }

    pub fn run(&mut self) {
        let event_loop = winit::event_loop::EventLoop::new().unwrap();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        event_loop.run_app(&mut self.window).unwrap();
    }
}
