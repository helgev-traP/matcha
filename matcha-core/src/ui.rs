use std::{any::Any, sync::Arc};

use super::{
    context::WidgetContext,
    events::Event,
    observer::Observer,
    renderer::RendererMap,
    types::range::CoverRange,
    vertex::{ColorVertex, UvVertex},
};

// dom tree node

#[async_trait::async_trait]
pub trait Dom<T>: Sync + Any {
    // if any dynamic widget is included in the widget tree, the second value is true.
    fn build_widget_tree(&self) -> Box<dyn Widget<T>>;
    async fn collect_observer(&self) -> Observer;
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
        context: &WidgetContext,
    ) -> Option<T>;

    // inside / outside check
    fn is_inside(
        &mut self,
        position: [f32; 2],
        parent_size: [Option<f32>; 2],
        context: &WidgetContext,
    ) -> bool {
        let px_size = self.px_size(parent_size, context);

        !(position[0] < 0.0
            || position[0] > px_size[0]
            || position[1] < 0.0
            || position[1] > px_size[1])
    }

    /// Actual size including its sub widgets with pixel value.
    fn px_size(&mut self, parent_size: [Option<f32>; 2], context: &WidgetContext) -> [f32; 2];

    /// The drawing range and the area that the widget always covers.
    fn cover_range(
        &mut self,
        parent_size: [Option<f32>; 2],
        context: &WidgetContext,
    ) -> CoverRange<f32>;

    fn updated(&self) -> bool;

    fn render(
        &mut self,
        parent_size: [Option<f32>; 2],
        background: Background,
        ctx: &WidgetContext,
    ) -> Vec<Object>;

    /// Updates the GPU device and queue for rendering purposes.
    /// This method is a placeholder and should be implemented to handle GPU resource updates.
    fn update_gpu_device(&mut self, _device: &wgpu::Device, _queue: &wgpu::Queue) {
        // TODO: Implement GPU device and queue updates.
        // This might involve updating buffers, textures, or other GPU resources.
    }
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

#[derive(Clone)]
pub enum Object<'a> {
    TextureColor {
        texture: Arc<wgpu::Texture>,
        uv_vertices: &'a [UvVertex],
        indices: &'a [u16],
        transform: nalgebra::Matrix4<f32>,
    },
    VertexColor {
        vertices: &'a [ColorVertex],
        indices: &'a [u16],
        transform: nalgebra::Matrix4<f32>,
    },
    // and more ...?
}

impl Object<'_> {
    pub fn transform(&mut self, affine: nalgebra::Matrix4<f32>) {
        match self {
            Object::TextureColor { transform, .. } | Object::VertexColor { transform, .. } => {
                *transform = affine * (*transform);
            }
        }
    }
}
