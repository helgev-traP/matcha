pub mod column;
pub mod teacup;

use nalgebra as na;
use std::{any::Any, sync::Arc};

use super::{
    application_context::ApplicationContext,
    events::WidgetEvent,
    types::size::{OptionPxSize, PxSize},
};

pub enum Object {
    NoObject,
    Textured {
        vertex_buffer: Arc<wgpu::Buffer>,
        index_buffer: Arc<wgpu::Buffer>,
        index_len: u32,
        texture: Arc<wgpu::Texture>,
    },
    Colored {
        vertex_buffer: Arc<wgpu::Buffer>,
        index_buffer: Arc<wgpu::Buffer>,
        index_len: u32,
    },
}

pub struct SubObject {
    pub affine: na::Matrix3<f32>,
    pub object: RenderObject,
}

pub struct RenderObject {
    object: Object,
    pub px_size: super::types::size::PxSize,
    // pub property: crate::ui::Property,
    pub sub_objects: Vec<SubObject>,
}

impl RenderObject {
    pub fn new(object: Object, px_size: PxSize, sub_objects: Vec<SubObject>) -> Self {
        Self {
            object,
            px_size,
            sub_objects,
        }
    }

    pub fn object(&mut self, frame: u64) -> &Object{
            &self.object
    }
}

pub trait DomNode<GlobalMessage>: Any + 'static {
    fn build_render_tree(&self) -> Box<dyn RenderNode<GlobalMessage>>;

    fn always_refresh(&self) -> bool {
        false
    }
}

pub trait RenderNode<GlobalMessage> {
    // for rendering
    fn render(
        &mut self,
        app_context: &ApplicationContext,
        parent_size: OptionPxSize,
    ) -> RenderObject;

    // widget event
    fn widget_event(&self, event: &WidgetEvent) -> Option<GlobalMessage>;
    // fn size(&self) -> OptionPxSize;
    // fn default_size(&self) -> PxSize;

    // for dom handling
    fn update_render_tree(&self, dom: &dyn DomNode<GlobalMessage>);
    fn compare(&self, dom: &dyn DomNode<GlobalMessage>) -> DomComPareResult;
}

pub trait Im {
    fn poll(&mut self) -> Object;
}

pub trait RenderNodeIm<GlobalMessage>: RenderNode<GlobalMessage> + Im {}

pub enum DomComPareResult {
    Same,
    Changed,
    Different,
}
