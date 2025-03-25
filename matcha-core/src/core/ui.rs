use std::{any::Any, sync::Arc};

use super::{
    context::SharedContext,
    events::{UiEvent, UiEventResult},
    renderer::Renderer,
    types::range::Range2D,
    vertex::uv_vertex::UvVertex,
};

// dom tree node

pub trait Dom<T>: Any {
    // if any dynamic widget is included in the widget tree, the second value is true.
    fn build_widget_tree(&self) -> Box<dyn Widget<T>>;
    // todo: consider use downcast-rs crate
    fn as_any(&self) -> &dyn Any;
}

// render tree node

#[derive(Debug, Clone, PartialEq)]
pub enum UpdateWidgetError {
    TypeMismatch,
}

// todo: consider integrate frame into `SharedContext`.
pub trait Widget<T> {
    // label
    fn label(&self) -> Option<&str>;

    // for dom handling
    fn update_widget_tree(&mut self, dom: &dyn Dom<T>) -> Result<(), UpdateWidgetError>;

    fn compare(&self, dom: &dyn Dom<T>) -> DomComPareResult;

    // widget event
    fn widget_event(
        &mut self,
        event: &UiEvent,
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
        tag: u64,
        frame: u64,
    ) -> UiEventResult<T>;

    // inside / outside check
    fn is_inside(
        &mut self,
        position: [f32; 2],
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
        tag: u64,
        frame: u64,
    ) -> bool {
        let px_size = self.px_size(parent_size, context, tag, frame);

        !(position[0] < 0.0
            || position[0] > px_size[0]
            || position[1] < 0.0
            || position[1] > px_size[1])
    }

    /// Actual size including its sub widgets with pixel value.
    fn px_size(
        &mut self,
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
        tag: u64,
        frame: u64,
    ) -> [f32; 2];

    /// The drawing range of the whole widget.
    fn draw_range(
        &mut self,
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
        tag: u64,
        frame: u64,
    ) -> Option<Range2D<f32>>;

    /// The area that the widget always covers.
    fn cover_area(
        &mut self,
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
        tag: u64,
        frame: u64,
    ) -> Option<Range2D<f32>>;

    fn redraw(&self) -> bool;

    // fn render(
    //     &mut self,
    //     // ui background
    //     parent_size: [Option<f32>; 2],
    //     background_view: &wgpu::TextureView,
    //     background_range: Range2D<f32>,
    //     // context
    //     context: &SharedContext,
    //     renderer: &Renderer,
    //     tag: u64,
    //     frame: u64,
    // ) -> Vec<Object>;

    fn render(
        &mut self,
        ui_background: UiBackground,
        ui_context: UiContext,
    ) -> Vec<Object>;

    // todo
    // fn update_gpu_device(&mut self, device: &wgpu::Device, queue: &wgpu::Queue);
}

#[derive(Clone, Copy)]
pub struct UiBackground<'a> {
    pub parent_size: [Option<f32>; 2],
    pub background_view: &'a wgpu::TextureView,
    pub background_range: Range2D<f32>,
}

#[derive(Clone, Copy)]
pub struct UiContext<'a> {
    pub context: &'a SharedContext,
    pub renderer: &'a Renderer,
    pub tag: u64,
    pub frame: u64,
}

pub enum DomComPareResult {
    Same,
    Changed,
    Different,
}

// todo: add object type when renderer is ready
// todo: Arcによる共有をObject内に持つべきか検討
// todo: add re-rendering range to apply scissors test for optimization
/// `Arc` is not necessary for sharing objects
/// since `Arc` is already used in this struct.
#[derive(Clone)]
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

impl Clone for TextureObject {
    fn clone(&self) -> Self {
        TextureObject {
            texture: Arc::clone(&self.texture),
            uv_vertices: Arc::clone(&self.uv_vertices),
            indices: Arc::clone(&self.indices),
            transform: self.transform.clone(),
        }
    }
}

pub struct TextureBlur {
    pub texture: Arc<wgpu::Texture>,
    pub uv_vertices: Arc<Vec<UvVertex>>,
    pub indices: Arc<Vec<u16>>,
    pub transform: nalgebra::Matrix4<f32>,
    pub blur: f32,
}

impl Clone for TextureBlur {
    fn clone(&self) -> Self {
        TextureBlur {
            texture: Arc::clone(&self.texture),
            uv_vertices: Arc::clone(&self.uv_vertices),
            indices: Arc::clone(&self.indices),
            transform: self.transform.clone(),
            blur: self.blur,
        }
    }
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
