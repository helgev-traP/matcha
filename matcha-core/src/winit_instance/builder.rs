use std::time::Duration;

use winit::dpi::PhysicalSize;

use crate::{
    Component,
    backend::Backend,
    device_input::mouse_state::MousePrimaryButton,
    types::color::Color,
    winit_instance::{
        AnyResource, WinitInstance, error, render_control, ticker, ui_control, window_surface,
    },
};

// --- Constants ---

// gpu
const POWER_PREFERENCE: wgpu::PowerPreference = wgpu::PowerPreference::LowPower;
const BASE_COLOR: Color = Color::TRANSPARENT;
const PREFERRED_SURFACE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
const COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
const STENCIL_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;

// input
const DOUBLE_CLICK_THRESHOLD: Duration = Duration::from_millis(300);
const LONG_PRESS_THRESHOLD: Duration = Duration::from_millis(500);
const SCROLL_PIXEL_PER_LINE: f32 = 40.0;
const DEFAULT_FONT_SIZE: f32 = 16.0;
const MOUSE_PRIMARY_BUTTON: MousePrimaryButton = MousePrimaryButton::Left;

// --- Builder ---

pub struct WinitInstanceBuilder<
    Model: Send + Sync + 'static,
    Message: 'static,
    B: Backend<Event> + Clone + 'static,
    Event: Send + 'static,
    InnerEvent: 'static = Event,
> {
    pub(crate) component: Component<Model, Message, Event, InnerEvent>,
    pub(crate) backend: B,
    pub(crate) runtime_builder: RuntimeBuilder,
    // window settings
    pub(crate) title: String,
    pub(crate) init_size: PhysicalSize<u32>,
    pub(crate) maximized: bool,
    pub(crate) full_screen: bool,
    // render settings
    pub(crate) power_preference: wgpu::PowerPreference,
    pub(crate) base_color: Color,
    pub(crate) surface_preferred_format: wgpu::TextureFormat,
    // input settings
    pub(crate) double_click_threshold: Duration,
    pub(crate) long_press_threshold: Duration,
    pub(crate) mouse_primary_button: MousePrimaryButton,
    pub(crate) scroll_pixel_per_line: f32,
    // font settings
    pub(crate) default_font_size: f32,
}

pub(crate) enum RuntimeBuilder {
    GivenRuntime(tokio::runtime::Runtime),
    CreateInternally { threads: usize },
}

impl RuntimeBuilder {
    pub fn build(self) -> Result<tokio::runtime::Runtime, std::io::Error> {
        match self {
            Self::GivenRuntime(runtime) => Ok(runtime),
            Self::CreateInternally { threads } => {
                let cpu_threads = std::thread::available_parallelism().map_or(1, |n| n.get());
                let threads = threads.min(cpu_threads);

                if threads == 1 {
                    tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                } else {
                    tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(threads)
                        .enable_all()
                        .build()
                }
            }
        }
    }
}

impl<
    Model: Send + Sync + 'static,
    Message: 'static,
    B: Backend<Event> + Clone + 'static,
    Event: Send + 'static,
    InnerEvent: 'static,
> WinitInstanceBuilder<Model, Message, B, Event, InnerEvent>
{
    pub fn new(component: Component<Model, Message, Event, InnerEvent>, backend: B) -> Self {
        let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
        Self {
            component,
            backend,
            runtime_builder: RuntimeBuilder::CreateInternally { threads },
            title: "Matcha App".to_string(),
            init_size: PhysicalSize::new(800, 600),
            maximized: false,
            full_screen: false,
            power_preference: POWER_PREFERENCE,
            base_color: BASE_COLOR,
            surface_preferred_format: PREFERRED_SURFACE_FORMAT,
            double_click_threshold: DOUBLE_CLICK_THRESHOLD,
            long_press_threshold: LONG_PRESS_THRESHOLD,
            mouse_primary_button: MOUSE_PRIMARY_BUTTON,
            scroll_pixel_per_line: SCROLL_PIXEL_PER_LINE,
            default_font_size: DEFAULT_FONT_SIZE,
        }
    }

    // --- Settings ---

    pub fn tokio_runtime(mut self, runtime: tokio::runtime::Runtime) -> Self {
        self.runtime_builder = RuntimeBuilder::GivenRuntime(runtime);
        self
    }

    pub fn worker_threads(mut self, threads: usize) -> Self {
        self.runtime_builder = RuntimeBuilder::CreateInternally { threads };
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn init_size(mut self, width: u32, height: u32) -> Self {
        self.init_size = PhysicalSize::new(width, height);
        self
    }

    pub fn maximized(mut self, maximized: bool) -> Self {
        self.maximized = maximized;
        self
    }

    pub fn full_screen(mut self, full_screen: bool) -> Self {
        self.full_screen = full_screen;
        self
    }

    pub fn power_preference(mut self, preference: wgpu::PowerPreference) -> Self {
        self.power_preference = preference;
        self
    }

    pub fn base_color(mut self, color: Color) -> Self {
        self.base_color = color;
        self
    }

    pub fn surface_preferred_format(mut self, format: wgpu::TextureFormat) -> Self {
        self.surface_preferred_format = format;
        self
    }

    pub fn double_click_threshold(mut self, duration: Duration) -> Self {
        self.double_click_threshold = duration;
        self
    }

    pub fn long_press_threshold(mut self, duration: Duration) -> Self {
        self.long_press_threshold = duration;
        self
    }

    pub fn mouse_primary_button(mut self, button: MousePrimaryButton) -> Self {
        self.mouse_primary_button = button;
        self
    }

    pub fn scroll_pixel_per_line(mut self, pixel: f32) -> Self {
        self.scroll_pixel_per_line = pixel;
        self
    }

    pub fn default_font_size(mut self, size: f32) -> Self {
        self.default_font_size = size;
        self
    }

    // --- Build ---

    pub fn build(
        self,
    ) -> Result<WinitInstance<Model, Message, B, Event, InnerEvent>, error::InitError> {
        let tokio_runtime = self
            .runtime_builder
            .build()
            .map_err(|_| error::InitError::TokioRuntime)?;

        let render_control = tokio_runtime
            .block_on(render_control::RenderControl::new(
                self.power_preference,
                self.base_color.into(),
                COLOR_FORMAT,
                STENCIL_FORMAT,
            ))
            .map_err(|_| error::InitError::Gpu)?;

        let ui_control = ui_control::UiControl::new(
            self.component,
            self.double_click_threshold,
            self.long_press_threshold,
            self.mouse_primary_button,
            self.scroll_pixel_per_line,
            self.default_font_size,
        )?;

        let mut window = window_surface::WindowSurface::new();
        window.set_title(self.title.as_str());
        window.set_init_size(self.init_size);
        window.set_maximized(self.maximized);
        if self.full_screen {
            window.set_fullscreen(true);
        }

        Ok(WinitInstance {
            tokio_runtime,
            window,
            surface_preferred_format: self.surface_preferred_format,
            any_resource: AnyResource::new(),
            render_control,
            ui_control,
            backend: self.backend,
            benchmarker: super::benchmark::Benchmark::new(60),
            ticker: ticker::Ticker::new(),
        })
    }
}
