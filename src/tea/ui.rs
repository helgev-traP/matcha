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
    // if any dynamic widget is included in the widget tree, the second value is true.
    fn build_widget_tree(&self) -> (Box<dyn Widget<T>>, bool);
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
    fn drawing_range(&self, parent_size: [StdSize; 2], context: &SharedContext) -> [[f32; 2]; 2];

    /// The area that the widget always covers.
    fn cover_area(&self, parent_size: [StdSize; 2], context: &SharedContext) -> Option<[[f32; 2]; 2]>;

    fn has_dynamic(&self) -> bool;

    fn redraw(&self) -> bool;

    fn render(
        &mut self,
        // ui environment
        parent_size: [StdSize; 2],
        background_view: &wgpu::TextureView,
        background_position: [[f32; 2]; 2], // [{upper left uv_x, uv_y}, {lower right uv_x, uv_y}]
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

impl Object {
    pub fn translate(&mut self, affine: nalgebra::Matrix4<f32>) {
        match self {
            Object::TextureObject(texture_object) => {
                texture_object.transform = affine * texture_object.transform;
            }
            Object::TextureBlur(texture_blur) => {
                texture_blur.transform = affine * texture_blur.transform;
            }
        }
    }
}
