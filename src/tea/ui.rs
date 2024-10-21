pub mod column;
pub mod drag_field;
pub mod row;
pub mod square;
pub mod teacup;
pub mod template;

use nalgebra as na;
use std::{any::Any, sync::Arc};

use super::{
    application_context::ApplicationContext,
    events::{UiEvent, UiEventResult},
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
    // label
    fn label(&self) -> Option<&str>;

    // for dom handling
    fn update_render_tree(&mut self, dom: &dyn Dom<GlobalMessage>) -> Result<(), ()>;
    fn compare(&self, dom: &dyn Dom<GlobalMessage>) -> DomComPareResult;

    // raw event
    // todo ?
    // fn raw_event(&self, event: ?) -> ?;

    // widget event
    fn widget_event(
        &mut self,
        event: &UiEvent,
        parent_size: PxSize,
        context: &ApplicationContext,
    ) -> UiEventResult<GlobalMessage>;

    // inside / outside check
    // todo
    fn is_inside(&self, position: [f32; 2], parent_size: PxSize, context: &ApplicationContext) -> bool;
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
