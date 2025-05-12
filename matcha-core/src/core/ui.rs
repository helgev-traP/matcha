use std::{any::Any, sync::Arc};

use super::{
    context::SharedContext,
    events::Event,
    observer::Observer,
    renderer::Renderer,
    types::range::CoverRange,
    vertex::uv_vertex::UvVertex,
};

// dom tree node

#[async_trait::async_trait]
pub trait Dom<T>: Sync + Any {
    // if any dynamic widget is included in the widget tree, the second value is true.
    fn build_widget_tree(&self) -> Box<dyn Widget<T>>;
    async fn collect_observer(&self) -> Observer;
    // todo: consider use downcast-rs crate
    fn as_any(&self) -> &dyn Any;
}

// render tree node

#[derive(Debug, Clone, PartialEq)]
pub enum UpdateWidgetError {
    TypeMismatch,
}

#[async_trait::async_trait]
pub trait Widget<T>: Send {
    // label
    fn label(&self) -> Option<&str>;

    // for dom handling
    async fn update_widget_tree(
        &mut self,
        component_updated: bool,
        dom: &dyn Dom<T>,
    ) -> Result<(), UpdateWidgetError>;

    fn compare(&self, dom: &dyn Dom<T>) -> DomComPareResult;

    // widget event
    fn widget_event(
        &mut self,
        event: &Event,
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
    ) -> Option<T>;

    // inside / outside check
    fn is_inside(
        &mut self,
        position: [f32; 2],
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
    ) -> bool {
        let px_size = self.px_size(parent_size, context);

        !(position[0] < 0.0
            || position[0] > px_size[0]
            || position[1] < 0.0
            || position[1] > px_size[1])
    }

    /// Actual size including its sub widgets with pixel value.
    fn px_size(&mut self, parent_size: [Option<f32>; 2], context: &SharedContext) -> [f32; 2];

    /// The drawing range and the area that the widget always covers.
    fn cover_range(
        &mut self,
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
    ) -> CoverRange<f32>;

    fn redraw(&self) -> bool;

    fn render(
        &mut self,
        parent_size: [Option<f32>; 2],
        background: Background,
        context: &SharedContext,
        renderer: &Renderer,
    ) -> Object;

    // todo
    // fn update_gpu_device(&mut self, device: &wgpu::Device, queue: &wgpu::Queue);
}

pub enum DomComPareResult {
    Same,
    Changed(usize),
    Different,
}

#[derive(Clone, Copy)]
pub struct Background<'a> {
    view: &'a wgpu::TextureView,
    position: [f32; 2],
}

impl<'a> Background<'a> {
    pub fn new(view: &'a wgpu::TextureView, position: [f32; 2]) -> Self {
        Self { view, position }
    }

    pub fn view(&self) -> &wgpu::TextureView {
        self.view
    }

    pub fn position(&self) -> [f32; 2] {
        self.position
    }

    pub fn transition(mut self, position: [f32; 2]) -> Self {
        self.position = [
            self.position[0] + position[0],
            self.position[1] + position[1],
        ];
        self
    }
}

// todo: add object type when renderer is ready
// todo: Arcによる共有をObject内に持つべきか検討
// todo: add re-rendering range to apply scissors test for optimization
/// `Arc` is not necessary for sharing objects
/// since `Arc` is already used in this struct.
pub struct Object {
    pub texture_color: Vec<TextureColor>,
    pub vertex_color: Vec<VertexColor>,
    // Gradation
    // GradationBlur ?
    // and more ...?
}

impl Object {
    pub fn transform(&mut self, affine: nalgebra::Matrix4<f32>) {
        for texture_color in &mut self.texture_color {
            texture_color.transform(affine);
        }
        for vertex_color in &mut self.vertex_color {
            vertex_color.transform(affine);
        }
    }

    pub fn add_texture_color(&mut self, obj: TextureColor) {
        self.texture_color.push(obj);
    }

    pub fn add_vertex_color(&mut self, obj: VertexColor) {
        self.vertex_color.push(obj);
    }
}

pub struct TextureColor {
    pub texture: Arc<wgpu::Texture>,
    pub uv_vertices: Vec<UvVertex>,
    pub indices: Vec<u16>,
    pub transform: nalgebra::Matrix4<f32>,
}

impl Clone for TextureColor {
    fn clone(&self) -> Self {
        Self {
            texture: Arc::clone(&self.texture),
            uv_vertices: self.uv_vertices.clone(),
            indices: self.indices.clone(),
            transform: self.transform,
        }
    }
}

impl TextureColor {
    pub fn transform(&mut self, affine: nalgebra::Matrix4<f32>) {
        self.transform = affine * self.transform;
    }
}

pub struct VertexColor {
    pub vertices: Vec<()>,
    pub indices: Vec<u16>,
    pub transform: nalgebra::Matrix4<f32>,
}

impl Clone for VertexColor {
    fn clone(&self) -> Self {
        Self {
            vertices: self.vertices.clone(),
            indices: self.indices.clone(),
            transform: self.transform,
        }
    }
}

impl VertexColor {
    pub fn transform(&mut self, affine: nalgebra::Matrix4<f32>) {
        self.transform = affine * self.transform;
    }
}
