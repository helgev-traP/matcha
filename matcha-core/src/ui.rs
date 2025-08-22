use std::{any::Any, time::Duration};

use crate::{
    any_resource::AnyResource,
    device_event::DeviceEvent,
    gpu::DeviceQueue,
    render_node::RenderNode,
    texture_allocator,
    types::range::{CoverRange, Range2D},
    update_flag::UpdateNotifier,
};

/// A struct that represents the constraints for a widget's size.
/// This is passed from parent to child to define the available space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Constraints {
    /// The minimum width the widget can have.
    pub min_width: f32,
    /// The maximum width the widget can have.
    pub max_width: f32,
    /// The minimum height the widget can have.
    pub min_height: f32,
    /// The maximum height the widget can have.
    pub max_height: f32,
}

impl Constraints {
    /// `[{min}, {max}]`
    pub fn new(width: [f32; 2], height: [f32; 2]) -> Self {
        Self {
            min_width: width[0],
            max_width: width[1],
            min_height: height[0],
            max_height: height[1],
        }
    }
}

/// Provides contextual information available to all widgets during their lifecycle.
///
/// This includes access to the GPU, window properties, shared resources, and timing information.
/// It is passed down the widget tree during layout and rendering.
#[derive(Clone)]
pub struct WidgetContext<'a> {
    device_queue: DeviceQueue<'a>,
    surface_format: wgpu::TextureFormat,
    window_size: [f32; 2],
    window_dpi: f64,
    texture_atlas: &'a texture_allocator::TextureAllocator,
    any_resource: &'a AnyResource,
    root_font_size: f32,
    font_size: f32,
    current_time: Duration,
}

impl<'a> WidgetContext<'a> {
    #[doc(hidden)]
    pub(crate) const fn new(
        device_queue: DeviceQueue<'a>,
        surface_format: wgpu::TextureFormat,
        window_size: [f32; 2],
        window_dpi: f64,
        texture_atlas: &'a texture_allocator::TextureAllocator,
        any_resource: &'a AnyResource,
        root_font_size: f32,
        current_time: Duration,
    ) -> Self {
        Self {
            device_queue,
            surface_format,
            window_size,
            window_dpi,
            texture_atlas,
            any_resource,
            root_font_size,
            font_size: root_font_size,
            current_time,
        }
    }

    /// Returns a reference to the WGPU device.
    pub fn device(&self) -> &wgpu::Device {
        self.device_queue.device()
    }

    /// Returns a reference to the WGPU queue.
    pub fn queue(&self) -> &wgpu::Queue {
        self.device_queue.queue()
    }

    /// Provides access to a type-safe, shared resource storage.
    pub fn any_resource(&self) -> &AnyResource {
        self.any_resource
    }

    /// Returns the texture format of the surface.
    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.surface_format
    }

    /// Returns the texture format for color used by the texture atlas.
    pub fn texture_format(&self) -> wgpu::TextureFormat {
        self.texture_atlas.color_format()
    }

    /// Returns a reference to the texture allocator.
    pub fn texture_atlas(&self) -> &texture_allocator::TextureAllocator {
        self.texture_atlas
    }

    /// Returns the texture format for stencil used by the texture atlas.
    pub fn stencil_format(&self) -> wgpu::TextureFormat {
        self.texture_atlas.stencil_format()
    }

    /// Returns the DPI scaling factor of the window.
    pub fn dpi(&self) -> f64 {
        self.window_dpi
    }

    /// Returns the logical size of the viewport.
    pub fn viewport_size(&self) -> [f32; 2] {
        self.window_size
    }

    /// Returns the current absolute time since the application started.
    pub fn current_time(&self) -> Duration {
        self.current_time
    }
}

impl WidgetContext<'_> {
    /// Returns the current font size.
    pub fn font_size(&self) -> f32 {
        self.font_size
    }

    /// Returns the root font size.
    pub fn root_font_size(&self) -> f32 {
        self.root_font_size
    }
}

impl WidgetContext<'_> {
    /// Creates a new context with a different font size.
    pub const fn with_font_size(&self, font_size: f32) -> Self {
        Self {
            device_queue: self.device_queue,
            surface_format: self.surface_format,
            window_size: self.window_size,
            window_dpi: self.window_dpi,
            texture_atlas: self.texture_atlas,
            any_resource: self.any_resource,
            root_font_size: self.root_font_size,
            font_size,
            current_time: self.current_time,
        }
    }
}

// dom tree node

/// Represents a node in the declarative UI tree (like a Virtual DOM).
///
/// The `Dom` tree is a stateless, declarative representation of the UI based on the application's `Model`.
/// Its primary responsibility is to build the stateful `Widget` tree.
/// Note: Coordinates used by the Dom/Widget/Style APIs use the top-left as the origin, the Y axis points downwards, and units are pixels.
#[async_trait::async_trait]
pub trait Dom<T>: Sync + Any {
    /// Builds the corresponding stateful `Widget` tree from this `Dom` node.
    fn build_widget_tree(&self) -> Box<dyn Widget<T>>;

    /// Sets an `UpdateNotifier` for the `Dom` tree to listen for model updates.
    ///
    /// This method is crucial for the `Component` system to detect changes in the `Model`.
    /// `ComponentDom` uses this to receive the notifier.
    ///
    /// Custom `Dom` implementations that contain children (e.g., layout widgets)
    /// have the responsibility to recursively propagate this notifier to all their children.
    /// Failure to do so will prevent descendant `Component`s from detecting model updates.
    async fn set_update_notifier(&self, notifier: &UpdateNotifier);
}

// Style

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

/// Represents an error that can occur when updating a `Widget` tree.
#[derive(Debug, Clone, PartialEq)]
pub enum UpdateWidgetError {
    /// Occurs when the type of the new `Dom` node does not match the existing `Widget`.
    TypeMismatch,
}

/// Represents a stateful node in the UI tree used for layout, event handling, and rendering.
///
/// The `Widget` tree is constructed from the `Dom` tree and holds the state of the UI,
/// including layout information and animation state. It follows a specific lifecycle for processing.
/// Note: Widget/Style APIs operate in a coordinate system with the origin at the top-left, the Y axis pointing downwards, and units in pixels.
///
/// # Lifecycle
///
/// 1.  **Measure (`preferred_size`)**: In this pass, the widget calculates its desired size based on the constraints
///     provided by its parent. This is a bottom-up process where children are measured before their parents.
/// 2.  **Arrange (`arrange`)**: In this pass, the parent widget determines the final size and position of its children
///     based on the results of the measure pass. This is a top-down process.
/// 3.  **Render (`render`)**: After the layout is determined, this pass generates the actual drawing commands
///     (`RenderNode`) to be sent to the GPU.
[#[async_trait::async_trait]]
pub trait Widget<T>: Send {
    /// Returns an optional label for debugging purposes.
    fn label(&self) -> Option<&str>;

    /// Updates the existing widget tree with a new `Dom` tree.
    ///
    /// This is part of the diffing algorithm to avoid rebuilding the entire tree on every update.
    async fn update_widget_tree(
        &mut self,
        component_updated: bool,
        dom: &dyn Dom<T>,
    ) -> Result<(), UpdateWidgetError>;

    /// Compares the widget with a `Dom` node to determine if they are compatible for an update.
    fn compare(&self, dom: &dyn Dom<T>) -> DomComPareResult;

    /// Processes a device event (e.g., mouse click, key press).
    ///
    /// Returns an application-specific event `T` if the event is handled.
    fn device_event(&mut self, event: &DeviceEvent, context: &WidgetContext) -> Option<T>;

    /// Checks if a given position is inside the widget's bounds.
    fn is_inside(&mut self, position: [f32; 2], context: &WidgetContext) -> bool;

    /// **Measure Pass**: Calculates the widget's preferred size based on the given constraints.
    fn preferred_size(&self, constraints: &Constraints, context: &WidgetContext) -> [f32; 2];

    /// **Arrange Pass**: Arranges the widget and its children within the given final size.
    fn arrange(&mut self, final_size: [f32; 2], context: &WidgetContext);

    // /// Returns the drawing range and the area that the widget always covers.
    // fn cover_range(&mut self, context: &WidgetContext) -> CoverRange<f32>;

    /// Indicates whether the widget needs to be re-rendered.
    fn need_rerendering(&self) -> bool;

    /// **Render Pass**: Generates a `RenderNode` for drawing.
    ///
    /// This method receives an `animation_update_flag_notifier` by value, allowing the widget
    /// to store it (e.g., in an `Option<UpdateNotifier>` field). The widget is then responsible
    /// for using this notifier to request a redraw whenever its internal state changes in a way
    /// that requires a visual update.
    ///
    /// This mechanism enables an efficient, reactive rendering loop where redraws only happen when
    /// explicitly requested by a widget, for example, due to an animation tick or a state change
    /// during event handling.
    fn render(&mut self, background: Background, ctx: &WidgetContext) -> RenderNode;

    /// Updates the GPU device and queue for rendering purposes.
    /// This method is a placeholder and should be implemented to handle GPU resource updates.
    fn update_gpu_device(&mut self, _device: &wgpu::Device, _queue: &wgpu::Queue) {
        // TODO: Implement GPU device and queue updates.
        // This might involve updating buffers, textures, or other GPU resources.
    }
}

/// The result of comparing a `Widget` with a `Dom` node.
pub enum DomComPareResult {
    /// The widget and DOM are of the same type and can be updated.
    Same,
    /// The widget and DOM are of the same type, but some properties have changed.
    Changed(usize),
    /// The widget and DOM are of different types and the widget must be rebuilt.
    Different,
}

/// Represents the background onto which a widget is rendered.
#[derive(Clone, Copy)]
pub struct Background<'a> {
    view: &'a wgpu::TextureView,
    position: [f32; 2],
}

impl<'a> Background<'a> {
    /// Creates a new `Background`.
    pub fn new(view: &'a wgpu::TextureView, position: [f32; 2]) -> Self {
        Self { view, position }
    }

    /// Returns the texture view of the background.
    pub fn view(&self) -> &wgpu::TextureView {
        self.view
    }

    /// Returns the current top-left position of the background.
    pub fn position(&self) -> [f32; 2] {
        self.position
    }

    /// Translates the background by a given position, returning a new `Background`.
    pub fn transition(mut self, position: [f32; 2]) -> Self {
        self.position = [
            self.position[0] + position[0],
            self.position[1] + position[1],
        ];
        self
    }
}
