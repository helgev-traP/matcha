pub mod column;
pub mod super_simple_button;
pub mod teacup;
pub mod template;

use nalgebra as na;
use std::{any::Any, sync::Arc};

use super::{
    application_context::ApplicationContext,
    events::{WidgetEvent, WidgetEventResult},
    renderer::RendererCommandEncoder,
    types::size::{PxSize, Size},
};

// render item

pub enum Object {
    Textured {
        instance_affine: na::Matrix4<f32>,
        vertex_buffer: Arc<wgpu::Buffer>,
        index_buffer: Arc<wgpu::Buffer>,
        index_len: u32,
        texture: Arc<wgpu::Texture>,
    },
    Colored {
        instance_affine: na::Matrix4<f32>,
        vertex_buffer: Arc<wgpu::Buffer>,
        index_buffer: Arc<wgpu::Buffer>,
        index_len: u32,
    },
}

pub struct RenderItem {
    pub object: Vec<Object>,
    // pub px_size: super::types::size::PxSize,
    // pub property: crate::ui::Property,
}

// dom tree node

pub trait Dom<Response>: Any + 'static {
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

pub trait WidgetTrait<GlobalMessage> {
    fn label(&self) -> Option<&str>;

    // widget event
    fn widget_event(
        &self,
        event: &WidgetEvent,
        parent_size: PxSize,
        Content: &ApplicationContext,
    ) -> WidgetEventResult<GlobalMessage>;

    // for dom handling
    fn update_render_tree(&mut self, dom: &dyn Dom<GlobalMessage>) -> Result<(), ()>;
    fn compare(&self, dom: &dyn Dom<GlobalMessage>) -> DomComPareResult;
}

pub enum DomComPareResult {
    Same,
    Changed,
    Different,
}

pub trait RenderingTrait: Send {
    /// The size configuration of the widget.
    fn size(&self) -> Size;

    /// Actual size including its sub widgets with pixel value.
    fn px_size(&self, parent_size: PxSize, context: &ApplicationContext) -> PxSize;

    /// Default size of widget with pixel value.
    fn default_size(&self) -> PxSize;

    fn render(
        &mut self,
        s: &rayon::Scope,
        parent_size: PxSize,
        affine: na::Matrix4<f32>,
        encoder: &mut RendererCommandEncoder,
    );
}
