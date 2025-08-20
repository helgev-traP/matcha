use std::sync::Arc;
use thiserror::Error;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event_loop::ActiveEventLoop,
    window::{Fullscreen, Window},
};

use crate::gpu::Gpu;

pub struct WindowSurface {
    window: Option<Arc<Window>>,
    surface: Option<wgpu::Surface<'static>>,
    surface_config: Option<wgpu::SurfaceConfiguration>,
    // window settings
    title: String,
    init_size: PhysicalSize<u32>,
    maximized: bool,
    fullscreen: bool,
}

impl WindowSurface {
    pub fn new() -> Self {
        Self {
            window: None,
            surface: None,
            surface_config: None,
            title: "Matcha App".to_string(),
            init_size: PhysicalSize::new(800, 600),
            maximized: false,
            fullscreen: false,
        }
    }

    // --- Settings ---

    pub fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
        if let Some(window) = &self.window {
            window.set_title(title);
        }
    }

    pub fn set_init_size(&mut self, size: PhysicalSize<u32>) {
        self.init_size = size;
    }

    pub fn set_maximized(&mut self, maximized: bool) {
        self.maximized = maximized;
        if let Some(window) = &self.window {
            window.set_maximized(maximized);
        }
    }

    pub fn set_fullscreen(&mut self, fullscreen: bool) {
        self.fullscreen = fullscreen;
        if let Some(window) = &self.window {
            if fullscreen {
                window.set_fullscreen(Some(Fullscreen::Borderless(None)));
            } else {
                window.set_fullscreen(None);
            }
        }
    }

    // --- Winit Integration ---

    pub fn start_window(
        &mut self,
        event_loop: &ActiveEventLoop,
        preferred_surface_format: wgpu::TextureFormat,
        gpu: &Gpu,
    ) -> Result<(), WindowSurfaceError> {
        let window_attributes = Window::default_attributes()
            .with_title(&self.title)
            .with_inner_size(self.init_size)
            .with_maximized(self.maximized);

        let window = Arc::new(event_loop.create_window(window_attributes)?);

        if self.fullscreen {
            window.set_fullscreen(Some(Fullscreen::Borderless(None)));
        }

        let surface = gpu.instance().create_surface(window.clone())?;

        let if_preferred_format_supported = surface
            .get_capabilities(gpu.adapter())
            .formats
            .contains(&preferred_surface_format);

        let mut surface_config = surface
            .get_default_config(
                gpu.adapter(),
                window.inner_size().width,
                window.inner_size().height,
            )
            .map(|mut config| {
                config.usage = wgpu::TextureUsages::RENDER_ATTACHMENT;
                config.present_mode = wgpu::PresentMode::AutoVsync;
                config.desired_maximum_frame_latency = 1;
                config.alpha_mode = wgpu::CompositeAlphaMode::Auto;
                config
            })
            .ok_or(WindowSurfaceError::SurfaceConfiguration)?;

        if if_preferred_format_supported {
            surface_config.format = preferred_surface_format;
        }

        surface.configure(gpu.device(), &surface_config);

        self.window = Some(window);
        self.surface = Some(surface);
        self.surface_config = Some(surface_config);

        Ok(())
    }

    // --- Getters ---

    pub fn window(&self) -> Option<&Arc<Window>> {
        self.window.as_ref()
    }

    pub fn get_current_texture(&self) -> Option<Result<wgpu::SurfaceTexture, wgpu::SurfaceError>> {
        self.surface.as_ref().map(|s| s.get_current_texture())
    }

    pub fn set_size(&mut self, size: PhysicalSize<u32>, device: &wgpu::Device) {
        if size.width == 0 || size.height == 0 {
            return;
        }

        if let (Some(surface), Some(surface_config)) = (&self.surface, &mut self.surface_config) {
            surface_config.width = size.width;
            surface_config.height = size.height;
            surface.configure(device, surface_config);
        }
    }

    pub fn format(&self) -> Option<wgpu::TextureFormat> {
        self.surface_config.as_ref().map(|c| c.format)
    }

    pub fn inner_size(&self) -> Option<PhysicalSize<u32>> {
        self.window.as_ref().map(|w| w.inner_size())
    }

    pub fn outer_size(&self) -> Option<PhysicalSize<u32>> {
        self.window.as_ref().map(|w| w.outer_size())
    }

    pub fn inner_position(
        &self,
    ) -> Option<Result<PhysicalPosition<i32>, winit::error::NotSupportedError>> {
        self.window.as_ref().map(|w| w.inner_position())
    }

    pub fn outer_position(
        &self,
    ) -> Option<Result<PhysicalPosition<i32>, winit::error::NotSupportedError>> {
        self.window.as_ref().map(|w| w.outer_position())
    }

    pub fn dpi(&self) -> Option<f64> {
        self.window.as_ref().map(|w| w.scale_factor())
    }

    pub fn request_redraw(&self) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

#[derive(Debug, Error)]
pub enum WindowSurfaceError {
    #[error(transparent)]
    Os(#[from] winit::error::OsError),
    #[error(transparent)]
    CreateSurface(#[from] wgpu::CreateSurfaceError),
    #[error("Failed to get surface configuration")]
    SurfaceConfiguration,
}
