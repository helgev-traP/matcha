// pub mod column;
pub mod teacup;

use nalgebra as na;
use std::{any::Any, sync::Arc};

use super::{
    application_context::ApplicationContext,
    events::WidgetEvent,
    types::size::{OptionPxSize, PxSize, Size},
};

// render object

pub enum Object {
    NoObject,
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
    object: Vec<Object>,
    pub px_size: super::types::size::PxSize,
    // pub property: crate::ui::Property,
}

impl RenderItem {
    pub fn new(object: Vec<Object>, px_size: PxSize) -> Self {
        Self { object, px_size }
    }

    pub fn object(&mut self) -> &Vec<Object> {
        &self.object
    }
}

pub struct SubNode<R> {
    pub affine: na::Matrix4<f32>,
    pub node: RenderNode<R>,
}

// dom tree node
pub trait DomNode<GlobalMessage>: Any + 'static {
    fn build_render_tree(&self) -> RenderNode<GlobalMessage>;
}

// render tree node
pub type RenderNode<GlobalMessage> = Box<dyn RenderTrait<GlobalMessage>>;

pub trait RenderTrait<GlobalMessage> {
    // for rendering
    fn sub_nodes(&self) -> Vec<SubNode<GlobalMessage>>;

    fn redraw(&self) -> bool {
        true
    }

    /// The size configuration of the widget.
    fn size(&self) -> Size;

    /// Actual size including its sub widgets with pixel value.
    fn px_size(&self, parent_size: PxSize, context: &ApplicationContext) -> PxSize;

    /// Default size of widget with pixel value.
    fn default_size(&self) -> PxSize;

    fn render(&mut self, app_context: &ApplicationContext, parent_size: PxSize) -> RenderItem;

    // widget event
    fn widget_event(&self, event: &WidgetEvent) -> Option<GlobalMessage>;

    // for dom handling
    fn update_render_tree(&self, dom: &dyn DomNode<GlobalMessage>);
    fn compare(&self, dom: &dyn DomNode<GlobalMessage>) -> DomComPareResult;
}

pub enum DomComPareResult {
    Same,
    Changed,
    Different,
}
