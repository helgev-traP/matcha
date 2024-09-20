use std::sync::Arc;

use crate::{application_context::ApplicationContext, cosmic::FontContext};

use super::{render::Renderer, types::PxSize, ui::TeaUi};

struct WindowState<'a> {
    instance: wgpu::Instance,
    adaptor: wgpu::Adapter,
    surface: wgpu::Surface<'a>,
    app_context: ApplicationContext,
    config: wgpu::SurfaceConfiguration,
}

impl<'a> WindowState<'a> {
    pub async fn new(
        winit_window: Arc<winit::window::Window>,
        power_preference: wgpu::PowerPreference,
        cosmic_context: Option<FontContext>,
    ) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(winit_window.clone()).unwrap();

        let adapter = instance
            .request_adapter(
                &(wgpu::RequestAdapterOptions {
                    power_preference,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                }),
            )
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &(wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web, we'll have to disable some.
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    memory_hints: wgpu::MemoryHints::default(),
                }),
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let size = winit_window.inner_size();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let app_context =
            ApplicationContext::new(winit_window, device, queue, surface_format, cosmic_context);

        Self {
            instance,
            adaptor: adapter,
            surface,
            app_context,
            config,
        }
    }

    pub fn get_app_context(&self) -> ApplicationContext {
        self.app_context.clone()
    }

    pub fn get_current_texture(&self) -> wgpu::SurfaceTexture {
        self.surface.get_current_texture().unwrap()
    }

    pub fn get_config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.app_context.device, &self.config);
    }
}

pub struct Window<'a> {
    winit_window: Option<Arc<winit::window::Window>>,
    window: Option<WindowState<'a>>,
    performance: wgpu::PowerPreference,
    title: String,
    init_size: [u32; 2],
    maximized: bool,
    full_screen: bool,

    cosmic_context: Option<crate::cosmic::FontContext>,

    render: Renderer,

    base_color: super::types::Color,
    uis: Vec<Box<dyn TeaUi>>,
}

impl Window<'_> {
    pub fn new() -> Self {
        Self {
            winit_window: None,
            window: None,
            performance: wgpu::PowerPreference::LowPower,
            title: "tea-ui".to_string(),
            init_size: [800, 600],
            maximized: false,
            full_screen: false,
            cosmic_context: None,
            render: Renderer::new(),
            base_color: super::types::Color::Rgba8USrgb {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            },
            uis: Vec::new(),
        }
    }

    pub fn performance(&mut self, performance: wgpu::PowerPreference) {
        self.performance = performance;
    }

    pub fn title(&mut self, title: &str) {
        self.title = title.to_string();
    }

    pub fn init_size(&mut self, width: u32, height: u32) {
        self.init_size = [width, height];
    }

    pub fn maximized(&mut self, maximized: bool) {
        self.maximized = maximized;
    }

    pub fn full_screen(&mut self, full_screen: bool) {
        self.full_screen = full_screen;
    }

    pub fn cosmic_context(&mut self, cosmic_context: crate::cosmic::FontContext) {
        self.cosmic_context = Some(cosmic_context);
    }

    pub fn base_color(&mut self, base_color: super::types::Color) {
        self.base_color = base_color;
    }

    pub fn ui(&mut self, uis: Vec<Box<dyn TeaUi>>) {
        self.uis = uis;
    }

    pub fn add_ui(&mut self, ui: Box<dyn TeaUi>) {
        self.uis.push(ui);
    }
}

impl winit::application::ApplicationHandler for Window<'_> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let mut winit_window = Arc::new(
            event_loop
                .create_window(winit::window::Window::default_attributes())
                .unwrap(),
        );

        winit_window.set_title(self.title.as_str());

        let _ = winit_window.request_inner_size(winit::dpi::PhysicalSize::new(
            self.init_size[0],
            self.init_size[1],
        ));

        if self.maximized {
            winit_window.set_maximized(true);
        }

        if self.full_screen {
            winit_window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
        }

        self.winit_window = Some(winit_window);

        let context = std::mem::take(&mut self.cosmic_context);

        let window_state = pollster::block_on(WindowState::new(
            self.winit_window.as_ref().unwrap().clone(),
            self.performance,
            context,
        ));

        self.window = Some(window_state);

        // give the render wgpu context

        self.render
            .set_application_context(self.window.as_ref().unwrap().get_app_context());

        for item in self.uis.iter_mut() {
            item.set_application_context(self.window.as_ref().unwrap().get_app_context());
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            winit::event::WindowEvent::RedrawRequested => {
                let size = super::types::PxSize {
                    width: self.window.as_ref().unwrap().get_config().width as f32,
                    height: self.window.as_ref().unwrap().get_config().height as f32,
                };
                let surface_texture = self.window.as_ref().unwrap().get_current_texture();

                self.render
                    .render(
                        surface_texture
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default()),
                        &size,
                        &self.base_color,
                        &self.uis,
                    )
                    .unwrap();

                surface_texture.present();
            }
            winit::event::WindowEvent::Resized(new_size) => {
                if new_size.width > 0 && new_size.height > 0 {
                    // update the surface configuration
                    self.window.as_mut().unwrap().resize(new_size);
                    // render
                    let size = super::types::PxSize {
                        width: self.window.as_ref().unwrap().get_config().width as f32,
                        height: self.window.as_ref().unwrap().get_config().height as f32,
                    };
                    let surface_texture = self.window.as_ref().unwrap().get_current_texture();

                    self.render
                        .render(
                            surface_texture
                                .texture
                                .create_view(&wgpu::TextureViewDescriptor::default()),
                            &size,
                            &self.base_color,
                            &self.uis,
                        )
                        .unwrap();

                    surface_texture.present();
                }
            }
            _ => {}
        }
    }

    fn new_events(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        cause: winit::event::StartCause,
    ) {
        let _ = (event_loop, cause);
    }

    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: ()) {
        let _ = (event_loop, event);
    }

    fn device_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let _ = (event_loop, device_id, event);
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let _ = event_loop;
    }

    fn suspended(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let _ = event_loop;
    }

    fn exiting(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let _ = event_loop;
    }

    fn memory_warning(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let _ = event_loop;
    }
}
