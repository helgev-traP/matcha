use std::{any::Any, sync::Arc};

use super::{
    context::SharedContext,
    events::{UiEvent, UiEventResult},
    types::size::{PxSize, Size}, vertex::textured_vertex::TexturedVertex,
};

// dom tree node

pub trait Dom<T>: Any + 'static {
    fn build_render_tree(&self) -> Box<dyn Widget<T>>;
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
        // ui environment
        parent_size: PxSize,
        // context
        context: &SharedContext,
    ) -> Vec<(
        Arc<wgpu::Texture>,
        Arc<Vec<TexturedVertex>>,
        Arc<Vec<u16>>,
        nalgebra::Matrix4<f32>,
    )> ;
}

pub enum DomComPareResult {
    Same,
    Changed,
    Different,
}
