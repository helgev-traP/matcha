pub mod column;
pub mod teacup;

use nalgebra as na;
use std::{any::Any, sync::Arc};

use super::{
    application_context::ApplicationContext,
    events::WidgetEvent,
    types::size::{OptionPxSize, PxSize},
};

pub enum Object<R> {
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
    TexturedIm {
        vertex_buffer: Arc<wgpu::Buffer>,
        index_buffer: Arc<wgpu::Buffer>,
        index_len: u32,
        texture: Arc<wgpu::Texture>,
        render_node: Arc<dyn RenderNode<R>>,
    },
    ColoredIm {
        vertex_buffer: Arc<wgpu::Buffer>,
        index_buffer: Arc<wgpu::Buffer>,
        index_len: u32,
    },
}

pub struct SubObject<R> {
    pub affine: na::Matrix3<f32>,
    pub object: RenderObject<R>,
}

pub struct RenderObject<R> {
    object: Object<R>,
    pub px_size: super::types::size::PxSize,
    // pub property: crate::ui::Property,
    pub sub_objects: Vec<SubObject<R>>,
}

impl<R> RenderObject<R> {
    pub fn new(object: Object<R>, px_size: PxSize, sub_objects: Vec<SubObject<R>>) -> Self {
        Self {
            object,
            px_size,
            sub_objects,
        }
    }

    pub fn object(&self) -> &Object<R>{
        match &self.object {
            Object::TexturedIm { .. } | Object::ColoredIm { .. } => {
                todo!()
            },
            _ => &self.object,
        }
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
    ) -> RenderObject<GlobalMessage>;

    // widget event
    fn widget_event(&self, event: &WidgetEvent) -> Option<GlobalMessage>;
    // fn size(&self) -> OptionPxSize;
    // fn default_size(&self) -> PxSize;

    // for dom handling
    fn update_render_tree(&self, dom: &dyn DomNode<GlobalMessage>);
    fn compare(&self, dom: &dyn DomNode<GlobalMessage>) -> DomComPareResult;
}

pub enum DomComPareResult {
    Same,
    Changed,
    Different,
}
