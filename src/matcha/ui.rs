use std::{any::Any, sync::Arc};
use vello::{wgpu, Scene};

use super::{
    context::SharedContext,
    events::{UiEvent, UiEventResult},
    types::size::{PxSize, Size},
};

// Texture layer

pub struct TextureLayer {
    pub texture: Arc<wgpu::Texture>,
    /// The position of the upper-left corner of the texture.
    ///
    /// Note that the **y-axis is upward** and the origin is the **upper-left** corner.
    pub position: [f32; 2],
    /// \[width, height\]
    pub size: [f32; 2],
}

pub struct LayerStack(Vec<TextureLayer>);

impl LayerStack {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, layer: TextureLayer) {
        self.0.push(layer);
    }

    pub fn extend(&mut self, layers: Vec<TextureLayer>) {
        self.0.extend(layers);
    }

    pub fn vec(self) -> Vec<TextureLayer> {
        self.0
    }
}

// dom tree node

pub trait Dom<Response>: Any + 'static {
    fn build_render_tree(&self) -> Box<dyn Widget<Response>>;
    fn as_any(&self) -> &dyn Any;
}

// render tree node

pub trait Widget<Response> {
    fn label(&self) -> Option<&str>;

    // --- update ---

    fn update_render_tree(&mut self, dom: &dyn Dom<Response>) -> Result<(), ()>;

    fn compare(&self, dom: &dyn Dom<Response>) -> DomComPareResult;

    // --- event handling ---

    fn event(
        &mut self,
        event: &UiEvent,
        parent_size: PxSize,
        context: &SharedContext,
    ) -> UiEventResult<Response>;

    fn is_inside(&self, position: [f32; 2], parent_size: PxSize, context: &SharedContext) -> bool;

    // --- rendering ---

    /// The size configuration of the widget.
    fn size(&self) -> Size;

    /// Actual size including its sub widgets with pixel value.
    fn px_size(&self, parent_size: PxSize, context: &SharedContext) -> PxSize;

    /// Default size of widget with pixel value.
    fn default_size(&self) -> PxSize;

    fn render(
        &mut self,
        scene: Option<&mut Scene>,
        texture_layer: &mut LayerStack,
        parent_size: PxSize,
        affine: vello::kurbo::Affine,
        context: &SharedContext,
    );
}

pub enum DomComPareResult {
    Same,
    Changed,
    Different,
}
