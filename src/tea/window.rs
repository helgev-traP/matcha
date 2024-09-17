use std::sync::Arc;

use crate::{application_context::ApplicationContext, cosmic::FontContext};

use super::render::Renderer;

pub struct WindowState<'a> {
    instance: wgpu::Instance,
    adaptor: wgpu::Adapter,
    surface: wgpu::Surface<'a>,
    app_context: ApplicationContext,
    config: wgpu::SurfaceConfiguration,
}

impl<'a> WindowState<'a> {
    async fn new(
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

    fn get_app_context(&self) -> ApplicationContext {
        self.app_context.clone()
    }

    fn get_current_texture(&self) -> wgpu::TextureView {
        self.surface
            .get_current_texture()
            .unwrap()
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default())
    }
}

struct Window<'a> {
    winit_window: Option<Arc<winit::window::Window>>,
    window: Option<WindowState<'a>>,
    performance: wgpu::PowerPreference,
    title: String,

    cosmic_context: Option<crate::cosmic::FontContext>,

    render: Renderer,
}

impl<'a> winit::application::ApplicationHandler for Window<'a> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.winit_window = Some(Arc::new(
            event_loop
                .create_window(winit::window::Window::default_attributes())
                .unwrap(),
        ));

        self.winit_window
            .as_ref()
            .unwrap()
            .set_title(self.title.as_str());

        let context = std::mem::take(&mut self.cosmic_context);

        let window_state = pollster::block_on(WindowState::new(
            self.winit_window.as_ref().unwrap().clone(),
            self.performance,
            context,
        ));

        self.window = Some(window_state);

        let size = self.winit_window.as_ref().unwrap().inner_size();

        // give the render wgpu context

        self.render
            .set_application_context(self.window.as_ref().unwrap().get_app_context());
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
                self.render
                    .render(self.window.as_ref().unwrap().get_current_texture());
            }
            winit::event::WindowEvent::Resized(size) => {}
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
