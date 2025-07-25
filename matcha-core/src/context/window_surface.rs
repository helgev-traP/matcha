use std::sync::Arc;

use winit::window::Window;

use crate::context::gpu::Gpu;

pub struct WindowSurface {
    window: Arc<winit::window::Window>,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
}

impl WindowSurface {
    pub fn new(window: Window, gpu: &Gpu) -> Self {
        let window = Arc::new(window);

        let surface = gpu.instance().create_surface(window.clone()).unwrap();

        let surface_caps = surface.get_capabilities(gpu.adapter());

        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let size = window.inner_size();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
        };

        surface.configure(gpu.device(), &config);

        Self {
            window,
            surface,
            config,
        }
    }

    pub fn window(&self) -> &winit::window::Window {
        &self.window
    }

    pub fn window_id(&self) -> winit::window::WindowId {
        self.window.id()
    }

    pub fn window_id_eq(&self, id: winit::window::WindowId) -> bool {
        self.window.id() == id
    }

    pub fn get_current_texture(&self) -> wgpu::SurfaceTexture {
        self.surface.get_current_texture().unwrap()
    }

    pub fn render_and_present<F>(&self, f: F)
    where
        F: FnOnce(wgpu::TextureView),
    {
        let frame = self.get_current_texture();
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        f(view);
        frame.present();
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>, gpu: &Gpu) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(gpu.device(), &self.config);
        }
    }

    pub fn config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    pub fn size(&self) -> [u32; 2] {
        [self.config.width, self.config.height]
    }

    pub fn dpi(&self) -> f64 {
        self.window.scale_factor()
    }
}
