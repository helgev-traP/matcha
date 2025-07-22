// This structure is for guaranteeing that textures are origin to same texture atlas.

use std::sync::Arc;

use parking_lot::Mutex;

use crate::context::gpu::Gpu;

pub struct RgbaTexture {
    inner: texture_atlas::Texture,
}

pub struct StencilTexture {
    inner: texture_atlas::Texture,
}

pub struct TextureAllocator {
    size: u32,
    rgba: Arc<Mutex<texture_atlas::TextureAtlas>>,
    stencil: Arc<Mutex<texture_atlas::TextureAtlas>>,
}

impl TextureAllocator {
    pub fn new(gpu: &Gpu) -> Self {
        let max_size = gpu.max_texture_dimension_3d();

        let size = wgpu::Extent3d {
            width: max_size,
            height: max_size,
            depth_or_array_layers: 1,
        };

        let rgba_atlas = texture_atlas::TextureAtlas::new(
            gpu.device(),
            size,
            &[wgpu::TextureFormat::Rgba8UnormSrgb],
        );

        let stencil_atlas =
            texture_atlas::TextureAtlas::new(gpu.device(), size, &[wgpu::TextureFormat::R32Uint]);

        Self {
            size: max_size,
            rgba: rgba_atlas,
            stencil: stencil_atlas,
        }
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn allocate_rgba(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: [u32; 2],
    ) -> Result<RgbaTexture, TextureAllocatorError> {
        let mut atlas = self.rgba.lock();
        let texture = atlas.allocate(device, queue, size)?;
        Ok(RgbaTexture { inner: texture })
    }
}

pub use texture_atlas::TextureAtlasError as TextureAllocatorError;
