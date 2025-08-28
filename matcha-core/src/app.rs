use super::{
    Component,
    backend::{Backend, StubBackend},
    device_event::mouse_state::MousePrimaryButton,
    types::color::Color,
    winit_instance::{WinitInstance, WinitInstanceBuilder},
};
use std::time::Duration;

pub struct App<Model, Message, B, Event, InnerEvent = Event>
where
    Model: Send + Sync + 'static,
    Message: 'static,
    B: Backend<Event> + Clone + 'static,
    Event: Send + 'static,
    InnerEvent: 'static,
{
    builder: WinitInstanceBuilder<Model, Message, B, Event, InnerEvent>,
}

impl<Model, Event, InnerEvent> App<Model, (), StubBackend, Event, InnerEvent>
where
    Model: Send + Sync + 'static,
    Event: std::fmt::Debug + Send + 'static,
    InnerEvent: 'static,
{
    pub fn new(component: Component<Model, (), Event, InnerEvent>) -> Self {
        Self {
            builder: WinitInstance::builder(component, StubBackend),
        }
    }
}

impl<Model, Message, B, Event, InnerEvent> App<Model, Message, B, Event, InnerEvent>
where
    Model: Send + Sync + 'static,
    Message: 'static,
    B: Backend<Event> + Clone + 'static,
    Event: std::fmt::Debug + Send + 'static,
    InnerEvent: 'static,
{
    pub fn with_backend<NewMessage, NewB>(
        self,
        component: Component<Model, NewMessage, Event, InnerEvent>,
        backend: NewB,
    ) -> App<Model, NewMessage, NewB, Event, InnerEvent>
    where
        NewMessage: 'static,
        NewB: Backend<Event> + Clone + 'static,
    {
        let mut new_builder = WinitInstance::builder(component, backend);
        // carry over settings
        new_builder.runtime_builder = self.builder.runtime_builder;
        new_builder.title = self.builder.title;
        new_builder.init_size = self.builder.init_size;
        new_builder.maximized = self.builder.maximized;
        new_builder.full_screen = self.builder.full_screen;
        new_builder.power_preference = self.builder.power_preference;
        new_builder.base_color = self.builder.base_color;
        new_builder.double_click_threshold = self.builder.double_click_threshold;
        new_builder.long_press_threshold = self.builder.long_press_threshold;
        new_builder.mouse_primary_button = self.builder.mouse_primary_button;
        new_builder.scroll_pixel_per_line = self.builder.scroll_pixel_per_line;
        new_builder.default_font_size = self.builder.default_font_size;

        App {
            builder: new_builder,
        }
    }

    pub fn tokio_runtime(mut self, runtime: tokio::runtime::Runtime) -> Self {
        self.builder = self.builder.tokio_runtime(runtime);
        self
    }

    pub fn worker_threads(mut self, threads: usize) -> Self {
        self.builder = self.builder.worker_threads(threads);
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.builder = self.builder.title(title);
        self
    }

    pub fn init_size(mut self, width: u32, height: u32) -> Self {
        self.builder = self.builder.init_size(width, height);
        self
    }

    pub fn maximized(mut self, maximized: bool) -> Self {
        self.builder = self.builder.maximized(maximized);
        self
    }

    pub fn full_screen(mut self, full_screen: bool) -> Self {
        self.builder = self.builder.full_screen(full_screen);
        self
    }

    pub fn power_preference(mut self, preference: wgpu::PowerPreference) -> Self {
        self.builder = self.builder.power_preference(preference);
        self
    }

    pub fn base_color(mut self, color: Color) -> Self {
        self.builder = self.builder.base_color(color);
        self
    }

    pub fn surface_preferred_format(mut self, format: wgpu::TextureFormat) -> Self {
        self.builder = self.builder.surface_preferred_format(format);
        self
    }

    pub fn double_click_threshold(mut self, duration: Duration) -> Self {
        self.builder = self.builder.double_click_threshold(duration);
        self
    }

    pub fn long_press_threshold(mut self, duration: Duration) -> Self {
        self.builder = self.builder.long_press_threshold(duration);
        self
    }

    pub fn mouse_primary_button(mut self, button: MousePrimaryButton) -> Self {
        self.builder = self.builder.mouse_primary_button(button);
        self
    }

    pub fn scroll_pixel_per_line(mut self, pixel: f32) -> Self {
        self.builder = self.builder.scroll_pixel_per_line(pixel);
        self
    }

    pub fn default_font_size(mut self, size: f32) -> Self {
        self.builder = self.builder.default_font_size(size);
        self
    }

    pub fn run(self) -> Result<(), winit::error::EventLoopError> {
        let mut winit_app = self.builder.build().expect("Failed to build WinitInstance");
        let event_loop = winit::event_loop::EventLoop::<Message>::with_user_event().build()?;
        event_loop.run_app(&mut winit_app)
    }
}
