use super::{panels::panel::InnerPanel, types::Size, ui::Widgets, window::Window};

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

    pub fn base_color(mut self, color: [u8; 4]) -> Self {
        self.window.get_top_panel().set_base_color(color);
        self
    }

    pub fn set_base_color(&mut self, color: [u8; 4]) {
        self.window.get_top_panel().set_base_color(color);
    }

    pub fn widgets(mut self, widgets: Vec<Box<dyn Widgets>>) -> Self {
        self.window.get_top_panel().set_widgets(widgets);
        self
    }

    pub fn panels(mut self, panels: Vec<InnerPanel>) -> Self {
        self.window.get_top_panel().set_panels(panels);
        self
    }

    pub fn add_widget(&mut self, widget: Box<dyn Widgets>) {
        self.window.get_top_panel().add_widget(widget);
    }

    pub fn add_top_panel(&mut self, thickness: f32) -> &mut InnerPanel {
        self.window.get_top_panel().add_top_panel(thickness)
    }

    pub fn add_bottom_panel(&mut self, thickness: f32) -> &mut InnerPanel {
        self.window.get_top_panel().add_bottom_panel(thickness)
    }

    pub fn add_left_panel(&mut self, thickness: f32) -> &mut InnerPanel {
        self.window.get_top_panel().add_left_panel(thickness)
    }

    pub fn add_right_panel(&mut self, thickness: f32) -> &mut InnerPanel {
        self.window.get_top_panel().add_right_panel(thickness)
    }

    pub fn add_floating_panel(&mut self, x: f32, y: f32, z: f32, size: Size) -> &mut InnerPanel {
        self.window
            .get_top_panel()
            .add_floating_panel(x, y, z, size)
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
