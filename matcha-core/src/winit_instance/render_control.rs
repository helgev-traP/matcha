use thiserror::Error;

use crate::{
    gpu::{DeviceQueue, Gpu, GpuError},
    renderer::{Renderer, TextureValidationError},
    texture_allocator::TextureAllocator,
    ui::Object,
};

pub struct RenderControl {
    gpu: Gpu,
    base_color: wgpu::Color,
    renderer: Renderer,
    texture_allocator: TextureAllocator,
}

impl RenderControl {
    pub async fn new(
        power_preferences: wgpu::PowerPreference,
        base_color: wgpu::Color,
        color_format: wgpu::TextureFormat,
        stencil_format: wgpu::TextureFormat,
    ) -> Result<Self, GpuError> {
        let gpu = Gpu::new(power_preferences).await?;
        let renderer = Renderer::new(gpu.device());

        let texture_allocator = TextureAllocator::new(&gpu, color_format, stencil_format);

        Ok(Self {
            gpu,
            base_color,
            renderer,
            texture_allocator,
        })
    }

    pub(crate) fn gpu(&self) -> &Gpu {
        &self.gpu
    }

    pub fn device(&self) -> &wgpu::Device {
        self.gpu.device()
    }

    pub fn queue(&self) -> &wgpu::Queue {
        self.gpu.queue()
    }

    pub fn device_queue(&self) -> DeviceQueue<'_> {
        self.gpu.device_queue()
    }

    pub fn texture_allocator(&self) -> &TextureAllocator {
        &self.texture_allocator
    }

    pub fn render(
        &self,
        object: Object,
        target_view: &wgpu::TextureView,
        viewport_size: [f32; 2],
        surface_format: wgpu::TextureFormat,
    ) -> Result<(), RenderControlError> {
        self.renderer
            .render(
                self.device(),
                self.queue(),
                surface_format,
                target_view,
                viewport_size,
                object,
                self.base_color,
                self.texture_allocator.color_texture(),
                self.texture_allocator.stencil_texture(),
            )
            .map_err(RenderControlError::TextureValidation)?;

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum RenderControlError {
    #[error("Texture validation error {0}")]
    TextureValidation(#[from] TextureValidationError),
}
