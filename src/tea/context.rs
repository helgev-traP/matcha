use std::sync::Arc;

use crate::cosmic::{FontContext, RenderAttribute, TextureAttribute, TextureAttributeGpu};

pub struct SharedContext {
    winit_window: Arc<winit::window::Window>,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    surface_format: wgpu::TextureFormat,

    cosmic_text: FontContext,
}

impl Clone for SharedContext {
    fn clone(&self) -> Self {
        Self {
            device: self.device.clone(),
            queue: self.queue.clone(),
            surface_format: self.surface_format,
            winit_window: self.winit_window.clone(),
            cosmic_text: self.cosmic_text.clone(),
        }
    }
}

impl SharedContext {
    pub fn new(
        winit_window: Arc<winit::window::Window>,
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        surface_format: wgpu::TextureFormat,
        cosmic_text: Option<FontContext>,
    ) -> Self {
        Self {
            winit_window,
            device: device,
            queue: queue,
            surface_format,
            cosmic_text: if let Some(cosmic_text) = cosmic_text {
                cosmic_text
            } else {
                FontContext::new()
            },
        }
    }

    pub fn get_wgpu_device(&self) -> &Arc<wgpu::Device> {
        &self.device
    }

    pub fn get_wgpu_queue(&self) -> &Arc<wgpu::Queue> {
        &self.queue
    }

    pub fn get_wgpu_encoder(&self) -> wgpu::CommandEncoder {
        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("App Context Command Encoder"),
            })
    }

    pub fn get_surface_format(&self) -> wgpu::TextureFormat {
        self.surface_format
    }

    pub fn get_dpi(&self) -> f64 {
        self.winit_window.scale_factor()
    }

    pub fn get_viewport_size(&self) -> (u32, u32) {
        let size = self.winit_window.inner_size();
        (size.width, size.height)
    }

    pub fn get_cosmic_text(&self) -> &FontContext {
        &self.cosmic_text
    }

    pub fn text_render(
        &self,
        text: &str,
        atr: RenderAttribute,
        texture: TextureAttribute,
    ) -> [i32; 2] {
        self.cosmic_text.render(
            text,
            atr,
            &TextureAttributeGpu {
                queue: &self.queue,
                width: texture.width,
                height: texture.height,
                texture: texture.texture,
            },
        )
    }
}
