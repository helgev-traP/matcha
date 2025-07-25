use std::sync::Arc;

use winit::{event_loop::ActiveEventLoop, window::Window};

use crate::context::gpu::Gpu;

pub struct WindowSurface {
    // winit
    window: Option<Arc<winit::window::Window>>,
    title: String,
    size: [u32; 2],
    init_maximized: bool,
    init_full_screen: bool,
    // wgpu
    surface: Option<wgpu::Surface<'static>>,
    config: wgpu::SurfaceConfiguration,
}

impl WindowSurface {
    pub fn new() -> Self {
        Self {
            window: None,
            title: String::new(),
            size: [800, 600],
            init_maximized: false,
            init_full_screen: false,
            surface: None,
            config: wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                width: 800,
                height: 600,
                present_mode: wgpu::PresentMode::AutoVsync,
                desired_maximum_frame_latency: 2,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![],
            },
        }
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
        if let Some(window) = self.window.as_ref() {
            window.set_title(&self.title);
        }
    }

    pub fn set_size(&mut self, new_size: winit::dpi::PhysicalSize<u32>, gpu: &Gpu) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = [new_size.width, new_size.height];
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            if let Some(surface) = &self.surface {
                surface.configure(gpu.device(), &self.config);
            }
        }
    }

    pub fn start_window(&mut self, event_loop: &ActiveEventLoop, gpu: &Gpu) {
        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_title("Matcha Window")
                    .with_inner_size(winit::dpi::PhysicalSize::new(self.size[0], self.size[1]))
                    .with_maximized(self.init_maximized)
                    .with_fullscreen(if self.init_full_screen {
                        Some(winit::window::Fullscreen::Borderless(None))
                    } else {
                        None
                    })
                    .with_resizable(true)
                    .with_visible(true),
            )
            .unwrap();

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
        self.config.format = surface_format;
        self.config.width = size.width;
        self.config.height = size.height;

        surface.configure(gpu.device(), &self.config);

        self.window = Some(window);
        self.surface = Some(surface);
    }

    pub fn window(&self) -> Option<&winit::window::Window> {
        self.window.as_ref().map(|v| &**v)
    }

    pub fn window_id(&self) -> Option<winit::window::WindowId> {
        self.window.as_ref().map(|w| w.id())
    }

    pub fn window_id_eq(&self, id: winit::window::WindowId) -> bool {
        self.window_id().map_or(false, |w| w == id)
    }

    pub fn get_current_texture(&self) -> Option<wgpu::SurfaceTexture> {
        self.surface
            .as_ref()
            .and_then(|s| s.get_current_texture().ok())
    }

    pub fn render_and_present<F>(&self, f: F)
    where
        F: FnOnce(wgpu::TextureView),
    {
        let Some(frame) = self.get_current_texture() else {
            return;
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        f(view);
        frame.present();
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

    pub fn dpi(&self) -> Option<f64> {
        self.window.as_ref().map(|w| w.scale_factor())
    }

    pub fn refresh_rate_millihertz(&self) -> Option<u32> {
        self.window
            .as_ref()
            .and_then(|w| w.current_monitor())
            .and_then(|m| m.refresh_rate_millihertz())
    }
}
