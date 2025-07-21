use std::{any::Any, sync::Arc};

use crate::{
    context::{WindowContext, any_resource::AnyResource, gpu::Gpu, window_surface::WindowSurface},
    types::range::Range2D,
};

use super::{events::Event, observer::Observer, types::range::CoverRange};

#[derive(Clone)]
pub struct WidgetContext<'a> {
    gpu: &'a Gpu,
    window_surface: &'a WindowSurface,
    window_context: &'a WindowContext,
    root_font_size: f32,
    font_size: f32,
}

impl<'a> WidgetContext<'a> {
    pub(crate) const fn new(
        gpu: &'a Gpu,
        window_surface: &'a WindowSurface,
        window_context: &'a WindowContext,
        root_font_size: f32,
    ) -> Self {
        Self {
            gpu,
            window_surface,
            window_context,
            root_font_size,
            font_size: root_font_size,
        }
    }

    pub fn device(&self) -> &Arc<wgpu::Device> {
        self.gpu.device()
    }

    pub fn queue(&self) -> &Arc<wgpu::Queue> {
        self.gpu.queue()
    }

    pub fn make_encoder(&self) -> wgpu::CommandEncoder {
        self.gpu
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Command Encoder created by WidgetContext"),
            })
    }

    pub fn any_resource(&self) -> &AnyResource {
        self.window_context.resource()
    }

    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.window_surface.format()
    }

    pub fn texture_format(&self) -> wgpu::TextureFormat {
        self.window_context.texture_format()
    }

    pub fn dpi(&self) -> f64 {
        self.window_surface.dpi()
    }

    pub fn viewport_size(&self) -> [u32; 2] {
        self.window_surface.size()
    }
}

impl WidgetContext<'_> {
    pub fn font_size(&self) -> f32 {
        self.font_size
    }

    pub fn root_font_size(&self) -> f32 {
        self.root_font_size
    }
}

impl WidgetContext<'_> {
    pub const fn with_font_size(&self, font_size: f32) -> Self {
        Self {
            gpu: self.gpu,
            window_surface: self.window_surface,
            window_context: self.window_context,
            root_font_size: self.root_font_size,
            font_size,
        }
    }
}

// dom tree node

#[async_trait::async_trait]
pub trait Dom<T>: Sync + Any {
    fn build_widget_tree(&self) -> Box<dyn Widget<T>>;
    async fn set_observer(&self, observer: &Observer);
}

// Style

pub trait Style: Send + Sync {
    fn clone_boxed(&self) -> Box<dyn Style>;
    /// is given position inside the shape of this style.
    fn is_inside(&self, position: [f32; 2], boundary_size: [f32; 2], ctx: &WidgetContext) -> bool;
    /// The y-axis in `Range2D` is points upward.
    fn draw_range(&self, boundary_size: [f32; 2], ctx: &WidgetContext) -> Range2D<f32>;
    /// The y-axis of `offset` is points upward.
    /// `offset` is the position of the upper left corner of the texture
    /// relative to the upper left corner of the boundary.
    fn draw(
        &self,
        render_pass: &mut wgpu::RenderPass<'_>,
        target_size: [u32; 2],
        target_format: wgpu::TextureFormat,
        boundary_size: [f32; 2],
        offset: [f32; 2],
        ctx: &WidgetContext,
    );
}

impl Clone for Box<dyn Style> {
    fn clone(&self) -> Self {
        self.clone_boxed()
    }
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

    // for dom handling]
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

    fn need_rerendering(&self) -> bool;

    fn render(
        &mut self,
        render_pass: &mut wgpu::RenderPass<'_>,
        target_size: [u32; 2],
        target_format: wgpu::TextureFormat,
        parent_size: [Option<f32>; 2],
        background: Background,
        ctx: &WidgetContext,
    ) -> Object;

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
pub struct Object {
    texture: (),
    texture_position: Range2D<f32>,
    stencil: Option<()>,
    stencil_position: Option<Range2D<f32>>,
    transform: nalgebra::Matrix4<f32>,

    child_elements: Vec<Object>,
}

impl Object {
    pub fn transform(&mut self, affine: nalgebra::Matrix4<f32>) {
        self.transform = affine * self.transform;
    }
}
