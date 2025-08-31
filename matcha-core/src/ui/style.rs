use super::WidgetContext;
use crate::types::range::Range2D;
use gpu_utils::texture_atlas::atlas_simple::atlas::AtlasRegion;

/// A trait that defines the visual appearance and drawing logic of a widget.
///
/// This allows for custom rendering logic to be encapsulated and reused.
pub trait Style: Send + Sync {
    /// Creates a clone of this style inside a `Box`.
    fn clone_boxed(&self) -> Box<dyn Style>;

    /// Calculates the minimum size required to draw this style.
    ///
    /// This method returns the intrinsic size of the visual content defined by the style,
    /// such as the dimensions of an image or the bounding box of a piece of text.
    /// The layout system may use this information to determine the widget's final size.
    ///
    /// # Returns
    ///
    /// An array `[width, height]` representing the required size in pixels.
    fn required_size(&self, ctx: &WidgetContext) -> Option<[f32; 2]> {
        let _ = ctx;
        None
    }

    /// Checks if a given position is inside the shape defined by this style.
    fn is_inside(&self, position: [f32; 2], boundary_size: [f32; 2], ctx: &WidgetContext) -> bool;

    /// Calculates the drawing range of the style.
    /// Coordinates are in pixels with the origin at the top-left and the Y axis pointing downwards.
    fn draw_range(&self, boundary_size: [f32; 2], ctx: &WidgetContext) -> Range2D<f32>;

    /// Draws the style onto the render pass.
    ///
    /// - `offset`: The position of the upper left corner of the texture relative to the upper left corner of the boundary.
    /// - Coordinates are in pixels; the origin is the upper-left of the boundary and the Y axis points downwards.
    fn draw(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &AtlasRegion,
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
