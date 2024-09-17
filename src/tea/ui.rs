// 
pub mod container_;
pub use container_::Container;

use nalgebra as na;

// for ui rendering
pub enum Object {
    Textured {
        vertex_buffer: Box<wgpu::Buffer>,
        index_buffer: Box<wgpu::Buffer>,
        index_len: u32,
        texture: Box<wgpu::Texture>,
    },
    Colored {
        vertex_buffer: Box<wgpu::Buffer>,
        index_buffer: Box<wgpu::Buffer>,
        index_len: u32,
    },
}

pub struct SubObject {
    pub affine: na::Matrix3<f32>,
    pub object: RenderObject,
}

pub struct RenderObject {
    pub object: Object,
    pub px_size: super::types::PxSize,
    pub overflow: bool,
    pub sub_objects: Vec<SubObject>,
}

pub trait UiRendering {
    fn render_object(&self) -> Result<RenderObject, ()>;
}

// for event handling
pub trait EventHandle {}

// for setting the Widget Style
pub trait Style {
}

// combine
pub trait TeaUi: UiRendering + EventHandle + Style {}

impl<T> TeaUi for T where T: UiRendering + EventHandle + Style {}
