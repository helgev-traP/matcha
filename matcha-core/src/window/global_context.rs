use std::sync::Arc;

use crate::renderer::{RendererSetup, RendererMap};

use super::context::WidgetContext;

pub struct GlobalContext<'a> {
    // gpu device
    _instance: wgpu::Instance,
    _adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,

    // winit window
    winit_window: Arc<winit::window::Window>,

    // wgpu surface
    config: wgpu::SurfaceConfiguration,
    surface: wgpu::Surface<'a>,
    surface_format: wgpu::TextureFormat,

    // common texture format
    texture_format: wgpu::TextureFormat,

    // custom renderers
    renderer_map: RendererMap,
}

impl GlobalContext<'_> {
    pub async fn new(
        winit_window: Arc<winit::window::Window>,
        power_preference: wgpu::PowerPreference,
        texture_format: wgpu::TextureFormat,
    ) -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
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
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

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

        Self {
            _instance: instance,
            _adapter: adapter,
            device,
            queue,
            winit_window,
            config,
            surface,
            surface_format,
            texture_format,
            renderer_map: RendererMap::new(),
        }
    }

    pub fn get_current_texture(&self) -> wgpu::SurfaceTexture {
        self.surface.get_current_texture().unwrap()
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 {
            // Update the surface configuration
            self.config.width = size.width;
            self.config.height = size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn add_renderer<T: RendererSetup>(&mut self, renderer: T) {
        self.renderer_map.add_only(renderer);
    }

    pub fn renderer_setup(&mut self) {
        self.renderer_map
            .setup(&self.device, &self.queue, self.surface_format);
    }
}

impl GlobalContext<'_> {
    pub fn widget_context(&self, font_size: f32) -> WidgetContext {
        WidgetContext::new(self, font_size)
    }

    pub fn renderer_map(&self) -> &RendererMap {
        &self.renderer_map
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn get_config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }

    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.surface_format
    }

    pub fn texture_format(&self) -> wgpu::TextureFormat {
        self.texture_format
    }

    pub fn dpi(&self) -> f64 {
        self.winit_window.scale_factor()
    }

    pub fn viewport_size(&self) -> [u32; 2] {
        // let size = self.winit_window.inner_size();
        [self.config.width, self.config.height]
    }
}
