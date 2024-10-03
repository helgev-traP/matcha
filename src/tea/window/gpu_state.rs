use std::sync::Arc;

use wgpu::core::device;

use crate::{cosmic::FontContext, types::size::PxSize};

use super::application_context::ApplicationContext;

pub struct GpuState<'a> {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    app_context: ApplicationContext,
    config: wgpu::SurfaceConfiguration,
    surface: wgpu::Surface<'a>,
}

impl GpuState<'_> {
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

        let app_context = ApplicationContext::new(
            winit_window,
            Arc::new(device),
            Arc::new(queue),
            surface_format,
            cosmic_context,
        );

        Self {
            instance,
            adapter,
            app_context,
            config,
            surface,
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

    pub fn get_viewport_size(&self) -> PxSize {
        PxSize {
            width: self.config.width as f32,
            height: self.config.height as f32,
        }
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 {
            self.config.width = size.width;
            self.config.height = size.height;
            self.surface
                .configure(&self.app_context.device, &self.config);
        }
    }
}
