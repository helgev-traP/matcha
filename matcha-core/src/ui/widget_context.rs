use std::time::Duration;

use crate::{
    any_resource::AnyResource, debug_config::SharedDebugConfig, gpu::DeviceQueue, texture_allocator,
};

/// Provides contextual information available to all widgets during their lifecycle.
///
/// This includes access to the GPU, window properties, shared resources, and timing information.
/// It is passed down the widget tree during layout and rendering.
#[derive(Clone)]
pub struct WidgetContext<'a> {
    device_queue: DeviceQueue<'a>,
    surface_format: wgpu::TextureFormat,
    window_size: [f32; 2],
    window_dpi: f64,
    texture_atlas: &'a texture_allocator::TextureAllocator,
    any_resource: &'a AnyResource,
    root_font_size: f32,
    font_size: f32,
    debug_config: SharedDebugConfig,
    current_time: Duration,
}

impl<'a> WidgetContext<'a> {
    pub(crate) fn new(
        device_queue: DeviceQueue<'a>,
        surface_format: wgpu::TextureFormat,
        window_size: [f32; 2],
        window_dpi: f64,
        texture_atlas: &'a texture_allocator::TextureAllocator,
        any_resource: &'a AnyResource,
        root_font_size: f32,
        debug_config: SharedDebugConfig,
        current_time: Duration,
    ) -> Self {
        Self {
            device_queue,
            surface_format,
            window_size,
            window_dpi,
            texture_atlas,
            any_resource,
            root_font_size,
            font_size: root_font_size,
            debug_config,
            current_time,
        }
    }

    /// Returns a reference to the WGPU device.
    pub fn device(&self) -> &wgpu::Device {
        self.device_queue.device()
    }

    /// Returns a reference to the WGPU queue.
    pub fn queue(&self) -> &wgpu::Queue {
        self.device_queue.queue()
    }

    /// Provides access to a type-safe, shared resource storage.
    pub fn any_resource(&self) -> &AnyResource {
        self.any_resource
    }

    /// Returns the texture format of the surface.
    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.surface_format
    }

    /// Returns the texture format for color used by the texture atlas.
    pub fn texture_format(&self) -> wgpu::TextureFormat {
        self.texture_atlas.color_format()
    }

    /// Returns a reference to the texture allocator.
    pub fn texture_atlas(&self) -> &texture_allocator::TextureAllocator {
        self.texture_atlas
    }

    /// Returns the texture format for stencil used by the texture atlas.
    pub fn stencil_format(&self) -> wgpu::TextureFormat {
        self.texture_atlas.stencil_format()
    }

    /// Returns the DPI scaling factor of the window.
    pub fn dpi(&self) -> f64 {
        self.window_dpi
    }

    /// Returns the logical size of the viewport.
    pub fn viewport_size(&self) -> [f32; 2] {
        self.window_size
    }

    /// Returns the current absolute time since the application started.
    pub fn current_time(&self) -> Duration {
        self.current_time
    }

    /// Returns a clone of the shared debug config.
    pub fn debug_config(&self) -> SharedDebugConfig {
        self.debug_config.clone()
    }
}

impl WidgetContext<'_> {
    /// Returns the current font size.
    pub fn font_size(&self) -> f32 {
        self.font_size
    }

    /// Returns the root font size.
    pub fn root_font_size(&self) -> f32 {
        self.root_font_size
    }
}

impl WidgetContext<'_> {
    /// Creates a new context with a different font size.
    pub fn with_font_size(&self, font_size: f32) -> Self {
        Self {
            device_queue: self.device_queue,
            surface_format: self.surface_format,
            window_size: self.window_size,
            window_dpi: self.window_dpi,
            texture_atlas: self.texture_atlas,
            any_resource: self.any_resource,
            root_font_size: self.root_font_size,
            font_size,
            debug_config: self.debug_config.clone(),
            current_time: self.current_time,
        }
    }
}
