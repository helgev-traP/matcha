pub mod any_resource;
pub mod gpu;
pub mod texture_allocator;
pub mod window_surface;

pub struct WindowContext {
    uniform_texture_format: wgpu::TextureFormat,
    atlas: texture_allocator::TextureAtlas,
    resource: any_resource::AnyResource,
}

impl WindowContext {
    pub fn new(format: wgpu::TextureFormat) -> Self {
        Self {
            uniform_texture_format: format,
            atlas: todo!(),
            resource: any_resource::AnyResource::new(),
        }
    }

    pub fn texture_format(&self) -> wgpu::TextureFormat {
        self.uniform_texture_format
    }

    pub fn atlas(&self) -> &texture_allocator::TextureAtlas {
        &self.atlas
    }

    pub fn resource(&self) -> &any_resource::AnyResource {
        &self.resource
    }
}
