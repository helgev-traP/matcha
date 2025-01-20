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

    /// The drawing range of the whole widget.
    fn drawing_range(&self) -> [[f32; 2]; 2];

    /// The area that the widget always covers.
    fn cover_area(&self) -> Option<[[f32; 2]; 2]>;

    fn has_dynamic(&self) -> bool;

    fn redraw(&self) -> bool;

    fn render(
        &mut self,
        // ui environment
        parent_size: [StdSize; 2],
        background_view: &wgpu::TextureView,
        // [{upper left x, y}, {lower right x, y}]
        background_position: [[f32; 2]; 2],
        // context
        context: &SharedContext,
        renderer: &Renderer,
        frame: u64,
    ) -> Vec<Object>;
}

pub enum DomComPareResult {
    Same,
    Changed,
    Different,
}

// todo: add object type when renderer is ready
pub enum Object {
    TextureObject(TextureObject),
    TextureBlur(TextureBlur),
    // Gradation
    // GradationBlur ?
    // and more ...?
}

pub struct TextureObject {
    pub texture: Arc<wgpu::Texture>,
    pub uv_vertices: Arc<Vec<UvVertex>>,
    pub indices: Arc<Vec<u16>>,
    pub transform: nalgebra::Matrix4<f32>,
}

pub struct TextureBlur {
    pub texture: Arc<wgpu::Texture>,
    pub uv_vertices: Arc<Vec<UvVertex>>,
    pub indices: Arc<Vec<u16>>,
    pub transform: nalgebra::Matrix4<f32>,
    pub blur: f32,
}
