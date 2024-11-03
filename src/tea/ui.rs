use nalgebra as na;
use std::any::Any;

use super::{
    application_context::ApplicationContext,
    events::{UiEvent, UiEventResult},
    renderer::RendererCommandEncoder,
    types::{
        color::Color,
        size::{PxSize, Size},
    },
};

// render item

pub enum Object<'a> {
    Textured {
        object_affine: na::Matrix4<f32>,
        vertex_buffer: &'a wgpu::Buffer,
        index_buffer: &'a wgpu::Buffer,
        index_len: u32,
        texture: &'a wgpu::Texture,
    },
    Colored {
        object_affine: na::Matrix4<f32>,
        vertex_buffer: &'a wgpu::Buffer,
        index_buffer: &'a wgpu::Buffer,
        index_len: u32,
        color: Color,
    },
}

pub struct RenderItem<'a> {
    pub object: Vec<Object<'a>>,
    // pub px_size: super::types::size::PxSize,
    // pub property: crate::ui::Property,
}

// dom tree node

pub trait Dom<Response>: Send + Any + 'static {
    fn build_render_tree(&self) -> Box<dyn Widget<Response>>;
    fn as_any(&self) -> &dyn Any;
}

// render tree node

pub trait Widget<Response>: WidgetTrait<Response> + RenderingTrait {
    fn for_rendering(&mut self) -> &mut dyn RenderingTrait;
}

impl<T, R> Widget<R> for T
where
    T: WidgetTrait<R> + RenderingTrait,
{
    fn for_rendering(&mut self) -> &mut dyn RenderingTrait {
        self
    }
}

#[async_trait::async_trait]
pub trait WidgetTrait<Response> {
    // label
    fn label(&self) -> Option<&str>;

    // for dom handling
    fn update_render_tree(&mut self, dom: &dyn Dom<Response>) -> Result<(), ()>;
    fn compare(&self, dom: &dyn Dom<Response>) -> DomComPareResult;

    // raw event
    // todo ?
    // fn raw_event(&self, event: ?) -> ?;

    // widget event
    async fn widget_event(
        &mut self,
        event: &UiEvent,
        parent_size: PxSize,
        context: &ApplicationContext,
    ) -> UiEventResult<Response>;

    // inside / outside check
    // todo
    async fn is_inside(
        &self,
        position: [f32; 2],
        parent_size: PxSize,
        context: &ApplicationContext,
    ) -> bool;
}

pub enum DomComPareResult {
    Same,
    Changed,
    Different,
}

#[async_trait::async_trait]
pub trait RenderingTrait: Send {
    /// The size configuration of the widget.
    async fn size(&self) -> Size;

    /// Actual size including its sub widgets with pixel value.
    async fn px_size(&self, parent_size: PxSize, context: &ApplicationContext) -> PxSize;

    /// Default size of widget with pixel value.
    async fn default_size(&self) -> PxSize;

    async fn render(
        &mut self,
        parent_size: PxSize,
        affine: na::Matrix4<f32>,
        encoder: RendererCommandEncoder,
    );
}
