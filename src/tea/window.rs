use cgmath::prelude::*;
use std::sync::Arc;
use wgpu::naga::proc::index;
use wgpu::{core::device, util::DeviceExt};
use winit::{self, event::Event, platform::run_on_demand::EventLoopExtRunOnDemand};

use crate::application_context::ApplicationContext;
use crate::cosmic::FontContext;
use crate::panels::panel::Panel;
use crate::types::Size;
use crate::ui::RenderArea;
use crate::ui::Ui;
use crate::ui::Widgets;

use super::vertex::TexturedVertex;
use super::widgets;
use super::widgets::teacup;

struct WindowState<'a> {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    surface: wgpu::Surface<'a>,
    app_context: ApplicationContext,
    config: wgpu::SurfaceConfiguration,
}

// - new
// - clone app_context
// - resize
// - render
impl<'a> WindowState<'a> {
    async fn new(
        winit_window: Arc<winit::window::Window>,
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
                    power_preference: wgpu::PowerPreference::HighPerformance,
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

        let app_context;

        if cosmic_context.is_none() {
            app_context = ApplicationContext::new(device, queue);
        } else {
            app_context =
                ApplicationContext::new_with_context(device, queue, cosmic_context.unwrap());
        }

        Self {
            instance,
            adapter,
            surface,
            app_context,
            config,
        }
    }

    fn clone_app_context(&self) -> ApplicationContext {
        self.app_context.clone()
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface
            .configure(&*self.app_context.get_wgpu_device(), &self.config);
    }

    fn get_surface_texture(&mut self) -> wgpu::SurfaceTexture {
        self.surface.get_current_texture().unwrap()
    }

    fn get_surface_format(&self) -> wgpu::TextureFormat {
        self.config.format
    }
}

pub struct Window<'a> {
    winit_window: Option<Arc<winit::window::Window>>,
    window: Option<WindowState<'a>>,
    top_panel: Panel,
    cosmic_context: Option<crate::cosmic::FontContext>,
}

impl<'a> Window<'a> {
    pub fn new() -> Self {
        Self {
            winit_window: None,
            window: None,
            top_panel: Panel::new_as_top(Size {
                width: -1.0,
                height: -1.0,
            }),
            cosmic_context: None,
        }
    }

    pub fn set_cosmic_context(&mut self, cosmic_context: crate::cosmic::FontContext) {
        self.cosmic_context = Some(cosmic_context);
    }

    pub fn get_top_panel(&mut self) -> &mut Panel {
        &mut self.top_panel
    }

    fn render(&mut self) {
        self.top_panel
            .render_to_surface(self.window.as_mut().unwrap().get_surface_texture());
    }
}

impl<'a> winit::application::ApplicationHandler for Window<'a> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.winit_window = Some(Arc::new(
            event_loop
                .create_window(winit::window::Window::default_attributes())
                .unwrap(),
        ));

        let context = std::mem::take(&mut self.cosmic_context);

        let window_state = pollster::block_on(WindowState::new(
            self.winit_window.as_ref().unwrap().clone(),
            context,
        ));

        self.window = Some(window_state);

        let size = self.winit_window.as_ref().unwrap().inner_size();

        self.top_panel
            .event(&crate::event::Event::Resize(crate::types::Size {
                width: size.width as f32,
                height: size.height as f32,
            }));

        self.top_panel
            .set_application_context_top_panel(self.window.as_ref().unwrap().clone_app_context(), self.window.as_ref().unwrap().get_surface_format());
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
                self.render();
            }
            winit::event::WindowEvent::Resized(new_size) => {
                if new_size.width > 0 && new_size.height > 0 {
                    self.window.as_mut().unwrap().resize(new_size);
                    self.top_panel
                        .event(&crate::event::Event::Resize(crate::types::Size {
                            width: new_size.width as f32,
                            height: new_size.height as f32,
                        }));
                }
            }
            _ => {}
        }
    }

    // ----------- The Optionals ------------

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
