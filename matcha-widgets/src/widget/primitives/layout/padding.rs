use std::any::Any;

use matcha_core::{
    context::SharedContext,
    events::{UiEvent, UiEventResult},
    types::range::Range2D,
    ui::{Dom, DomComPareResult, UiBackground, UiContext, UpdateWidgetError, Widget},
};

pub struct Padding<T>
where
    T: Send + 'static,
{
    // label
    label: Option<String>,

    // properties
    top: f32,
    right: f32,
    bottom: f32,
    left: f32,

    // content
    content: Option<Box<dyn Dom<T>>>,
}

impl<T> Default for Padding<T>
where
    T: Send + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

// constructor
impl<T> Padding<T>
where
    T: Send + 'static,
{
    pub fn new() -> Self {
        Self {
            label: None,
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
            content: None,
        }
    }

    pub fn label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    pub fn top(mut self, top: f32) -> Self {
        self.top = top;
        self
    }

    pub fn right(mut self, right: f32) -> Self {
        self.right = right;
        self
    }

    pub fn bottom(mut self, bottom: f32) -> Self {
        self.bottom = bottom;
        self
    }

    pub fn left(mut self, left: f32) -> Self {
        self.left = left;
        self
    }

    pub fn horizontal(mut self, horizontal: f32) -> Self {
        self.left = horizontal;
        self.right = horizontal;
        self
    }

    pub fn vertical(mut self, vertical: f32) -> Self {
        self.top = vertical;
        self.bottom = vertical;
        self
    }

    pub fn content(mut self, content: Box<dyn Dom<T>>) -> Self {
        self.content = Some(content);
        self
    }
}

impl<T> Dom<T> for Padding<T>
where
    T: Send + 'static,
{
    fn build_widget_tree(&self) -> Box<dyn Widget<T>> {
        Box::new(PaddingNode {
            label: self.label.clone(),
            top: self.top,
            right: self.right,
            bottom: self.bottom,
            left: self.left,
            content: self
                .content
                .as_ref()
                .map(|content| content.build_widget_tree()),
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct PaddingNode<T>
where
    T: Send + 'static,
{
    // label
    label: Option<String>,

    // properties
    top: f32,
    right: f32,
    bottom: f32,
    left: f32,

    // content
    content: Option<Box<dyn Widget<T>>>,
}

impl<T> Widget<T> for PaddingNode<T>
where
    T: Send + 'static,
{
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn update_widget_tree(
        &mut self,
        dom: &dyn Dom<T>,
    ) -> Result<(), UpdateWidgetError> {
        if let Some(dom) = dom.as_any().downcast_ref::<Padding<T>>() {
            // update properties
            self.label = dom.label.clone();
            self.top = dom.top;
            self.right = dom.right;
            self.bottom = dom.bottom;
            self.left = dom.left;

            // update content
            if let Some(dom_content) = &dom.content {
                if let Some(self_content) = self.content.as_mut() {
                    self_content.update_widget_tree(dom_content.as_ref())?;
                } else {
                    self.content = Some(dom_content.build_widget_tree());
                }
            } else {
                self.content = None;
            }

            Ok(())
        } else {
            Err(UpdateWidgetError::TypeMismatch)
        }
    }

    fn compare(&self, dom: &dyn Dom<T>) -> DomComPareResult {
        if let Some(dom) = dom.as_any().downcast_ref::<Padding<T>>() {
            todo!()
        } else {
            DomComPareResult::Different
        }
    }

    fn widget_event(
        &mut self,
        event: &UiEvent,
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
        tag: u64,
        frame: u64,
    ) -> UiEventResult<T> {
        // todo !
        UiEventResult::default()
    }

    fn px_size(
        &mut self,
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
        tag: u64,
        frame: u64,
    ) -> [f32; 2] {
        match parent_size {
            [Some(width), Some(height)] => [width, height],
            _ => {
                let content_op_size = [
                    parent_size[0].map(|v| v - self.left - self.right),
                    parent_size[1].map(|v| v - self.top - self.bottom),
                ];

                let content_size = self
                    .content
                    .as_mut()
                    .map(|content| content.px_size(content_op_size, context, tag, frame))
                    .unwrap_or([0.0, 0.0]);

                // todo: witch is better?
                // [
                //     self.left + content_size[0] + self.right,
                //     self.top + content_size[1] + self.bottom,
                // ]

                // or

                [
                    parent_size[0].unwrap_or(content_size[0] + self.left + self.right),
                    parent_size[1].unwrap_or(content_size[1] + self.top + self.bottom),
                ]
            }
        }
    }

    fn draw_range(
        &mut self,
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
        tag: u64,
        frame: u64,
    ) -> Option<Range2D<f32>> {
        let content_op_size = [
            parent_size[0].map(|v| v - self.left - self.right),
            parent_size[1].map(|v| v - self.top - self.bottom),
        ];

        let draw_range = self
            .content
            .as_mut()
            .and_then(|content| content.draw_range(content_op_size, context, tag, frame));

        draw_range.map(|draw_range| draw_range.slide([self.left, self.top]))
    }

    fn cover_area(
        &mut self,
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
        tag: u64,
        frame: u64,
    ) -> Option<Range2D<f32>> {
        let content_op_size = [
            parent_size[0].map(|v| v - self.left - self.right),
            parent_size[1].map(|v| v - self.top - self.bottom),
        ];

        let cover_area = self
            .content
            .as_mut()
            .and_then(|content| content.cover_area(content_op_size, context, tag, frame));

        cover_area.map(|cover_area| cover_area.slide([self.left, self.top]))
    }

    fn redraw(&self) -> bool {
        self.content
            .as_ref()
            .map(|content| content.redraw())
            .unwrap_or(false)
    }

    fn render(
        &mut self,
        ui_background: UiBackground,
        ui_context: UiContext,
    ) -> Vec<matcha_core::ui::Object> {
        self.content
            .as_mut()
            .map(|content| content.render(ui_background, ui_context))
            .unwrap_or_default()
            .into_iter()
            .map(|mut object| {
                object.translate(nalgebra::Matrix4::new_translation(&nalgebra::Vector3::new(
                    self.left, self.top, 0.0,
                )));
                object
            })
            .collect::<Vec<_>>()
    }
}
