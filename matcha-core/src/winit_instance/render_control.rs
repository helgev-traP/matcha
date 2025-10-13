use thiserror::Error;

use crate::gpu::GpuError;
use renderer::{
    core_renderer::CoreRenderer,
    // debug_renderer::DebugRenderer as CoreRenderer,
    core_renderer::TextureValidationError,
    render_node::RenderNode,
};

pub struct RenderControl {
    base_color: wgpu::Color,
    renderer: CoreRenderer,
}

impl RenderControl {
    pub async fn new(device: &wgpu::Device, base_color: wgpu::Color) -> Result<Self, GpuError> {
        let renderer = CoreRenderer::new(device);
        Ok(Self {
            base_color,
            renderer,
        })
    }

    pub fn render(
        &self,
        // target
        target_view: &wgpu::TextureView,
        viewport_size: [f32; 2],
        surface_format: wgpu::TextureFormat,
        // resources
        object: &RenderNode,
        texture_atlas: &wgpu::Texture,
        stencil_atlas: &wgpu::Texture,
        // gpu
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<(), RenderControlError> {
        self.renderer
            .render(
                device,
                queue,
                surface_format,
                target_view,
                viewport_size,
                object,
                self.base_color,
                texture_atlas,
                stencil_atlas,
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
