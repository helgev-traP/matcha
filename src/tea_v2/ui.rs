//
pub mod layout;
pub use layout::Container;
pub mod row;
pub use row::Row;
pub mod column;
pub use column::Column;
pub mod property;
pub use property::Property;

use nalgebra as na;

use super::types::ParentPxSize;

// for ui rendering
pub enum Object<'a> {
    NoObject,
    Textured {
        vertex_buffer: Box<wgpu::Buffer>,
        index_buffer: Box<wgpu::Buffer>,
        index_len: u32,
        texture: Box<&'a wgpu::Texture>,
    },
    Colored {
        vertex_buffer: Box<wgpu::Buffer>,
        index_buffer: Box<wgpu::Buffer>,
        index_len: u32,
    },
}

pub struct SubObject<'a> {
    pub affine: na::Matrix3<f32>,
    pub object: RenderObject<'a>,
}

pub struct RenderObject<'a> {
    pub object: Object<'a>,
    pub px_size: super::types::PxSize,
    pub property: Property,
    pub sub_objects: Vec<SubObject<'a>>,
}

pub trait UiRendering {
    fn set_application_context(
        &mut self,
        app_context: super::application_context::ApplicationContext,
    );
    fn get_style(&self) -> &Property;
    fn set_style(&mut self, property: Property);
    // fn render_necessity(&self) -> bool;
    fn render_object(&self, parent_size: ParentPxSize) -> Result<RenderObject, ()>;
}

// for event handling
pub trait EventHandle {}

// for setting the Widget Style
#[allow(unused_variables)]
pub trait Style {
    fn position(&mut self, position: property::Position) {}
    fn overflow(&mut self, overflow: property::Overflow) {}
    fn size(&mut self, size: super::types::Size) {}
    fn margin(&mut self, margin: property::Margin) {}
    fn padding(&mut self, padding: property::Padding) {}
    fn border(&mut self, border: property::Border) {}
    fn box_sizing(&mut self, box_sizing: property::BoxSizing) {}
    fn text_color(&mut self, text_color: super::types::Color) {}
    fn background_color(&mut self, background_color: super::types::Color) {}
    fn font_family(&mut self, font_family: String) {}
    fn font_size(&mut self, font_size: f32) {}
    fn line_height_em(&mut self, line_height_em: f32) {}
    fn letter_spacing(&mut self, letter_spacing: f32) {}
    fn font_weight(&mut self, font_weight: u32) {}
    fn font_style(&mut self, font_style: property::FontStyle) {}
    fn text_align(&mut self, text_align: property::TextAlign) {}
    fn text_decoration(&mut self, text_decoration: property::TextDecoration) {}
    fn opacity(&mut self, opacity: u8) {}
    fn visibility(&mut self, visibility: property::Visibility) {}
    fn cursor(&mut self, cursor: property::Cursor) {}
}

// combine
pub trait TeaUi: UiRendering + EventHandle + Style {}

impl<T: UiRendering + EventHandle + Style> TeaUi for T {}
