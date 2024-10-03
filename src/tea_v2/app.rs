use winit::event_loop;

use super::application_context::ApplicationContext;
use super::render::Renderer;
use super::ui::TeaUi;

pub struct App<'a> {
    // model
    // update
    // view
    window: super::window::Window<'a>,
}

impl App<'_> {
    pub fn new() -> Self {
        Self {
            window: super::window::Window::new(),
        }
    }

    pub fn performance(mut self, performance: wgpu::PowerPreference) -> Self {
        self.window.performance(performance);
        self
    }

    pub fn title(mut self, title: &str) -> Self {
        self.window.title(title);
        self
    }

    pub fn init_size(mut self, width: u32, height: u32) -> Self {
        self.window.init_size(width, height);
        self
    }

    pub fn maximized(mut self, maximized: bool) -> Self {
        self.window.maximized(maximized);
        self
    }

    pub fn full_screen(mut self, full_screen: bool) -> Self {
        self.window.full_screen(full_screen);
        self
    }

    pub fn cosmic_context(mut self, cosmic_context: crate::cosmic::FontContext) -> Self {
        self.window.cosmic_context(cosmic_context);
        self
    }

    pub fn run(&mut self) {
        let event_loop = event_loop::EventLoop::new().unwrap();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
        event_loop.run_app(&mut self.window).unwrap();
    }

    pub fn base_color(mut self, base_color: super::types::Color) -> Self {
        self.window.base_color(base_color);
        self
    }

    pub fn ui(mut self, uis: Vec<Box<dyn TeaUi>>) -> Self {
        self.window.ui(uis);
        self
    }

    pub fn add_ui(&mut self, ui: Box<dyn TeaUi>) {
        self.window.add_ui(ui);
    }
}

