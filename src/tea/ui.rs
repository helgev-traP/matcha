use std::{any::Any, sync::Arc};

use super::{
    context::SharedContext,
    events::{UiEvent, UiEventResult},
    renderer::Renderer,
    types::size::{Size, StdSize},
    vertex::uv_vertex::UvVertex,
};

// dom tree node

pub trait Dom<T>: Any + 'static {
    fn build_widget_tree(&self) -> Box<dyn Widget<T>>;
    fn as_any(&self) -> &dyn Any;
}

// render tree node

pub trait Widget<T> {
    // label
    fn label(&self) -> Option<&str>;

    // for dom handling
    fn update_widget_tree(&mut self, dom: &dyn Dom<T>) -> Result<(), ()>;
    fn compare(&self, dom: &dyn Dom<T>) -> DomComPareResult;

    // raw event
    // todo ?
    // fn raw_event(&self, event: ?) -> ?;

    // widget event
    fn widget_event(
        &mut self,
        event: &UiEvent,
        parent_size: [StdSize; 2],
        context: &SharedContext,
    ) -> UiEventResult<T>;

    // inside / outside check
    fn is_inside(
        &self,
        position: [f32; 2],
        parent_size: [StdSize; 2],
        context: &SharedContext,
    ) -> bool {
        let px_size = self.px_size(parent_size, context);

        !(position[0] < 0.0
            || position[0] > px_size[0]
            || position[1] < 0.0
            || position[1] > px_size[1])
    }

    /// The size configuration of the widget.
    fn size(&self) -> [Size; 2];

    /// Actual size including its sub widgets with pixel value.
    fn px_size(&self, parent_size: [StdSize; 2], context: &SharedContext) -> [f32; 2];

    /// Default size of widget with pixel value.
    // todo: this may should be removed. keep it for now.
    fn default_size(&self) -> [f32; 2];

    fn render(
        &mut self,
        // ui environment
        parent_size: [StdSize; 2],
        // context
        context: &SharedContext,
        renderer: &Renderer,
        frame: u64,
    ) -> Vec<(
        Arc<wgpu::Texture>,
        Arc<Vec<UvVertex>>,
        Arc<Vec<u16>>,
        nalgebra::Matrix4<f32>,
    )>;
}

pub enum DomComPareResult {
    Same,
    Changed,
    Different,
}
