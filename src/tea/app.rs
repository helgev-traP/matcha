use super::application_context::ApplicationContext;
use super::render::Renderer;
use super::window::WindowState;

pub struct App<'a> {
    // model
    // update
    // view
    window: Option<WindowState<'a>>,
    render: Renderer,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        Self {
            window: None,
            render: Renderer::new(),
        }
    }
}
