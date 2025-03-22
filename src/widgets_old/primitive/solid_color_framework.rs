use crate::{
    context::SharedContext,
    events::{UiEvent, UiEventResult},
    renderer::Renderer,
    types::{color::Color, range::Range2D, size::Size},
    ui::{Dom, DomComPareResult, Object, Widget},
    widgets::{
        framework::FrameworkNode,
        style::{Border, BoxSizing, Padding, Visibility},
    },
};

pub struct SolidColor<T: Send + 'static> {
    // label
    pub label: Option<String>,
    // properties
    pub size: [Size; 2],
    pub padding: Padding,
    pub box_sizing: BoxSizing,
    pub visibility: Visibility,
    // border painting
    pub border_shape: Border,
    pub border_color: Color,
    // background painting
    pub background_color: Color,
    // content
    pub content: Option<Box<dyn Dom<T>>>,
}

struct Properties {
    size: [Size; 2],
    padding: Padding,
    box_sizing: BoxSizing,
    visibility: Visibility,
    border_shape: Border,
    border_color: Color,
    background_color: Color,
}

struct Content<T> {
    content: Option<Box<dyn Widget<T>>>,
}

struct SettingsCache {}

struct ContextCache {}

impl<T: Send + 'static> Dom<T> for SolidColor<T> {
    fn build_widget_tree(&self) -> Box<dyn Widget<T>> {
        Box::new(FrameworkNode::new(
            self.label.as_deref(),
            Properties {
                size: self.size,
                padding: self.padding,
                box_sizing: self.box_sizing,
                visibility: self.visibility,
                border_shape: self.border_shape,
                border_color: self.border_color,
                background_color: self.background_color,
            },
            Content {
                content: self
                    .content
                    .as_ref()
                    .map(|content| content.build_widget_tree()),
            },
            update_widget_tree,
            compare,
            widget_event,
            size_calc,
            draw_range,
            cover_area,
            render,
        ))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// MARK: Widget functions

fn update_widget_tree<T: Send + 'static>(
    dom: &dyn Dom<T>,
    label: &mut Option<String>,
    properties: &mut Properties,
    content: &mut Content<T>,
    settings_cache: &mut Option<SettingsCache>,
) -> Result<bool, ()> {
    todo!("update_widget_tree")
}

fn compare<T: Send + 'static>(
    dom: &dyn Dom<T>,
    label: Option<&str>,
    properties: &Properties,
    content: &Content<T>,
    settings_cache: Option<&SettingsCache>,
) -> DomComPareResult {
    todo!("compare")
}

fn widget_event<T: Send + 'static>(
    ui_event: &UiEvent,
    parent_size: [Option<f32>; 2],
    shared_context: &SharedContext,
) -> UiEventResult<T> {
    todo!("widget_event")
}

fn size_calc<T: Send + 'static>(
    parent_size: [Option<f32>; 2],
    shared_context: &SharedContext,
    properties: &Properties,
    content: &mut Content<T>,
    settings_cache: &mut Option<SettingsCache>,
) -> ([f32; 2], [Option<f32>; 2]) {
    todo!("size_calc")
}

fn draw_range<T: Send + 'static>(
    actual_size: [f32; 2],
    current_op_size: [Option<f32>; 2],
    context_cache: &mut Option<ContextCache>,
    properties: &Properties,
    content: &mut Content<T>,
    settings_cache: &mut Option<SettingsCache>,
) -> Option<Range2D<f32>> {
    todo!("draw_range")
}

fn cover_area<T: Send + 'static>(
    actual_size: [f32; 2],
    current_op_size: [Option<f32>; 2],
    context_cache: &mut Option<ContextCache>,
    properties: &Properties,
    content: &mut Content<T>,
    settings_cache: &mut Option<SettingsCache>,
) -> Option<Range2D<f32>> {
    todo!("cover_area")
}

fn render<T: Send + 'static>(
    actual_size: [f32; 2],
    current_op_size: [Option<f32>; 2],
    context_cache: &mut Option<ContextCache>,
    properties: &Properties,
    content: &mut Content<T>,
    settings_cache: &mut Option<SettingsCache>,
    background_view: &wgpu::TextureView,
    background_range: Range2D<f32>,
    shared_context: &SharedContext,
    renderer: &Renderer,
) -> Vec<Object> {
    todo!("render")
}



