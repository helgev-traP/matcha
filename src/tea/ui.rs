pub mod teacup;

use nalgebra as na;
use std::{any::Any, sync::Arc};

use super::{application_context::ApplicationContext, events::WidgetEvent};

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
    pub object: Object,
    pub px_size: super::types::size::PxSize,
    // pub property: crate::ui::Property,
    pub sub_objects: Vec<SubObject>,
}

pub trait DomNode: Any {
    fn build_render_tree(&self) -> Box<dyn RenderNode>;

    fn always_refresh(&self) -> bool {
        false
    }
}

pub trait RenderNode {
    // for rendering
    fn render(&mut self, app_context: &ApplicationContext) -> RenderObject;

    // widget event
    fn widget_event(&self, event: &WidgetEvent);

    // for dom handling
    fn update_render_tree(&self, dom: &dyn DomNode);
    fn compare(&self, dom: &dyn DomNode) -> DomComPareResult;
}

pub enum DomComPareResult {
    Same,
    Changed,
    Different,
}
