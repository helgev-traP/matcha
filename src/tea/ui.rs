pub mod button;
pub mod column;
pub mod teacup;

use nalgebra as na;
use std::{
    any::Any,
    sync::Arc,
};

use super::{
    application_context::ApplicationContext,
    events::{WidgetEvent, WidgetEventResult},
    types::size::{PxSize, Size},
};

// render item

pub enum Object {
    Textured {
        affine: na::Matrix4<f32>,
        vertex_buffer: Arc<wgpu::Buffer>,
        index_buffer: Arc<wgpu::Buffer>,
        index_len: u32,
        texture: Arc<wgpu::Texture>,
    },
    Colored {
        affine: na::Matrix4<f32>,
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

// sub node

pub struct SubNode<'a> {
    pub affine: na::Matrix4<f32>,
    pub node: &'a dyn RenderingTrait,
}

// dom tree node

pub trait DomNode<Response>: Any + 'static {
    fn build_render_tree(&self) -> Box<dyn Widget<Response>>;
    fn as_any(&self) -> &dyn Any;
}

// render tree node

pub trait Widget<Response>: WidgetTrait<Response> + RenderingTrait {
    fn for_rendering(&self) -> &dyn RenderingTrait;
}

impl<T, R> Widget<R> for T where T: WidgetTrait<R> + RenderingTrait {
    fn for_rendering(&self) -> &dyn RenderingTrait {
        self
    }
}

pub trait WidgetTrait<GlobalMessage> {
    // widget event
    fn widget_event(&self, event: &WidgetEvent) -> WidgetEventResult<GlobalMessage>;

    // for dom handling
    fn update_render_tree(&mut self, dom: &dyn DomNode<GlobalMessage>) -> Result<(), ()>;
    fn compare(&self, dom: &dyn DomNode<GlobalMessage>) -> DomComPareResult;
}

pub enum DomComPareResult {
    Same,
    Changed,
    Different,
}

pub trait RenderingTrait {
    // for rendering
    fn sub_nodes(
        &self,
        parent_size: PxSize,
        context: &ApplicationContext,
    ) -> Vec<SubNode>;

    fn redraw(&self) -> bool {
        // return true here if current widget need to be redrawn
        true
    }

    fn redraw_sub(&self, parent_size: PxSize, context: &ApplicationContext) -> bool {
        self.sub_nodes(parent_size, context)
            .iter()
            .any(|sub_node| sub_node.node.redraw())
    }

    /// The size configuration of the widget.
    fn size(&self) -> Size;

    /// Actual size including its sub widgets with pixel value.
    fn px_size(&self, parent_size: PxSize, context: &ApplicationContext) -> PxSize;

    /// Default size of widget with pixel value.
    fn default_size(&self) -> PxSize;

    fn render(&self, app_context: &ApplicationContext, parent_size: PxSize) -> RenderItem;
}
