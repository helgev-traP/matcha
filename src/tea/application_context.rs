use std::sync::Arc;

use crate::cosmic::{FontContext, RenderAttribute, TextureAttribute, TextureAttributeGpu};

pub struct ApplicationContext {
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,

    cosmic_text: FontContext,
}

impl Clone for ApplicationContext {
    fn clone(&self) -> Self {
        Self {
            device: self.device.clone(),
            queue: self.queue.clone(),
            cosmic_text: self.cosmic_text.clone(),
        }
    }
}

impl ApplicationContext {
    pub fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        Self {
            device: Arc::new(device),
            queue: Arc::new(queue),
            cosmic_text: FontContext::new(),
        }
    }

    pub fn new_with_context(
        device: wgpu::Device,
        queue: wgpu::Queue,
        cosmic_text: FontContext,
    ) -> Self {
        Self {
            device: Arc::new(device),
            queue: Arc::new(queue),
            cosmic_text,
        }
    }

    pub fn get_wgpu_device(&self) -> &Arc<wgpu::Device> {
        &self.device
    }

    pub fn get_wgpu_device_clone(&self) -> Arc<wgpu::Device> {
        self.device.clone()
    }

    pub fn get_wgpu_queue(&self) -> &Arc<wgpu::Queue> {
        &self.queue
    }

    pub fn get_wgpu_queue_clone(&self) -> Arc<wgpu::Queue> {
        self.queue.clone()
    }

    pub fn get_wgpu_encoder(&self) -> wgpu::CommandEncoder {
        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Device Queue Command Encoder"),
            })
    }

    pub fn get_cosmic_text(&self) -> &FontContext {
        &self.cosmic_text
    }

    pub fn text_render(&mut self, text: &str, atr: &RenderAttribute, texture: &TextureAttribute) {
        self.cosmic_text.render(
            text,
            atr,
            &TextureAttributeGpu {
                queue: &self.queue,
                width: texture.width,
                height: texture.height,
                texture: texture.texture,
            },
        );
    }
}
