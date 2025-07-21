use super::{component::Component, types::color::Color, winit_instance::WinitInstance};

pub struct App<Model, Message, Response, IR = Response>
where
    Model: Send + Sync + 'static,
    Message: 'static,
    Response: 'static,
    IR: 'static,
{
    winit_app: WinitInstance<Model, Message, Response, IR>,
}

impl<Model, Message, Response, IR> App<Model, Message, Response, IR>
where
    Model: Send + Sync + 'static,
    Message: 'static,
    Response: std::fmt::Debug + 'static,
    IR: 'static,
{
    pub fn new(component: Component<Model, Message, Response, IR>) -> Self {
        Self {
            winit_app: WinitInstance::new(component),
        }
    }

    pub fn title(mut self, title: &str) -> Self {
        self.winit_app.title(title);
        self
    }

    pub fn base_color(mut self, color: Color) -> Self {
        self.winit_app.base_color(color);
        self
    }

    pub fn run(&mut self) {
        let event_loop = winit::event_loop::EventLoop::with_user_event()
            .build()
            .unwrap();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        let _ = event_loop.run_app(&mut self.winit_app);
    }
}
