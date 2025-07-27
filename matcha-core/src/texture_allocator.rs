// This structure is for guaranteeing that textures are origin to same texture atlas.

use std::sync::Arc;

use parking_lot::Mutex;

use super::gpu::Gpu;

pub struct TextureAllocator {
    size: u32,
    color_format: wgpu::TextureFormat,
    color: Arc<Mutex<texture_atlas::TextureAtlas>>,
    stencil_format: wgpu::TextureFormat,
    stencil: Arc<Mutex<texture_atlas::TextureAtlas>>,
}

impl TextureAllocator {
    pub fn new(
        gpu: &Gpu,
        color_format: wgpu::TextureFormat,
        stencil_format: wgpu::TextureFormat,
    ) -> Self {
        let max_size = gpu.max_texture_dimension_3d();

        let size = wgpu::Extent3d {
            width: max_size,
            height: max_size,
            depth_or_array_layers: 1,
        };

        let rgba_atlas = texture_atlas::TextureAtlas::new(gpu.device(), size, &[color_format]);

        let stencil_atlas = texture_atlas::TextureAtlas::new(gpu.device(), size, &[stencil_format]);

        Self {
            size: max_size,
            color_format,
            color: rgba_atlas,
            stencil_format,
            stencil: stencil_atlas,
        }
    }

    pub fn color_format(&self) -> wgpu::TextureFormat {
        self.color_format
    }

    pub fn stencil_format(&self) -> wgpu::TextureFormat {
        self.stencil_format
    }
}

impl TextureAllocator {
    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn allocate_color(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: [u32; 2],
    ) -> Result<texture_atlas::Texture, TextureAllocatorError> {
        self.color.lock().allocate(device, queue, size)
    }

    pub fn allocate_stencil(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: [u32; 2],
    ) -> Result<texture_atlas::Texture, TextureAllocatorError> {
        self.stencil.lock().allocate(device, queue, size)
    }

    pub(super) fn color_texture(&self) -> wgpu::Texture {
        self.color.lock().textures()[0].clone()
    }

    pub(super) fn stencil_texture(&self) -> wgpu::Texture {
        self.stencil.lock().textures()[0].clone()
    }
}

pub use texture_atlas::TextureAtlasError as TextureAllocatorError;
