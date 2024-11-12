use std::any::Any;

use super::{
    context::SharedContext,
    events::{UiEvent, UiEventResult},
    types::size::{PxSize, Size},
};

// set of texture and stencil

pub struct TextureSet {
    pub texture: wgpu::Texture,
    pub stencil: wgpu::Texture,
}

// dom tree node

pub trait Dom<Response>: Any + 'static {
    fn build_render_tree(&self) -> Box<dyn Widget<Response>>;
    fn as_any(&self) -> &dyn Any;
}

// render tree node

pub trait Widget<T> {
    // label
    fn label(&self) -> Option<&str>;

    // for dom handling
    fn update_render_tree(&mut self, dom: &dyn Dom<T>) -> Result<(), ()>;
    fn compare(&self, dom: &dyn Dom<T>) -> DomComPareResult;

    // raw event
    // todo ?
    // fn raw_event(&self, event: ?) -> ?;

    // widget event
    fn widget_event(
        &mut self,
        event: &UiEvent,
        parent_size: PxSize,
        context: &SharedContext,
    ) -> UiEventResult<T>;

    // inside / outside check
    // todo
    fn is_inside(&self, position: [f32; 2], parent_size: PxSize, context: &SharedContext) -> bool;

    /// The size configuration of the widget.
    fn size(&self) -> Size;

    /// Actual size including its sub widgets with pixel value.
    fn px_size(&self, parent_size: PxSize, context: &SharedContext) -> PxSize;

    /// Default size of widget with pixel value.
    fn default_size(&self) -> PxSize;

    fn render(
        &mut self,
        texture: Option<&TextureSet>,
        parent_size: PxSize,
        affine: nalgebra::Matrix4<f32>,
        context: &SharedContext,
    );
}

pub enum DomComPareResult {
    Same,
    Changed,
    Different,
}
