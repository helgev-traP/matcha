use std::any::Any;

use matcha_core::{
    common_resource::CommonResource,
    context::WidgetContext,
    device_event::DeviceEvent,
    observer::Observer,
    types::range::{CoverRange, Range2D},
    ui::{Background, Dom, DomComPareResult, UpdateWidgetError, Widget},
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

#[async_trait::async_trait]
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

    async fn set_observer(&self) -> Observer {
        if let Some(content) = &self.content {
            content.set_observer().await
        } else {
            Observer::default()
        }
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

#[async_trait::async_trait]
impl<T> Widget<T> for PaddingNode<T>
where
    T: Send + 'static,
{
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    async fn update_widget_tree(
        &mut self,
        component_updated: bool,
        dom: &dyn Dom<T>,
    ) -> Result<(), UpdateWidgetError> {
        if let Some(dom) = (dom as &dyn Any).downcast_ref::<Padding<T>>() {
            // update properties
            self.label = dom.label.clone();
            self.top = dom.top;
            self.right = dom.right;
            self.bottom = dom.bottom;
            self.left = dom.left;

            // update content
            if let Some(dom_content) = &dom.content {
                if let Some(self_content) = self.content.as_mut() {
                    self_content
                        .update_widget_tree(component_updated, dom_content.as_ref())
                        .await?;
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
        if let Some(dom) = (dom as &dyn Any).downcast_ref::<Padding<T>>() {
            todo!()
        } else {
            DomComPareResult::Different
        }
    }

    fn device_event(
        &mut self,
        event: &DeviceEvent,
        parent_size: [Option<f32>; 2],
        context: &WidgetContext,
    ) -> Option<T> {
        // todo !
        None
    }

    fn px_size(&mut self, parent_size: [Option<f32>; 2], context: &WidgetContext) -> [f32; 2] {
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
                    .map(|content| content.px_size(content_op_size, context))
                    .unwrap_or([0.0, 0.0]);

                [
                    parent_size[0].unwrap_or(content_size[0] + self.left + self.right),
                    parent_size[1].unwrap_or(content_size[1] + self.top + self.bottom),
                ]
            }
        }
    }

    // The drawing range and the area that the widget always covers.
    fn cover_range(
        &mut self,
        parent_size: [Option<f32>; 2],
        context: &WidgetContext,
    ) -> CoverRange<f32> {
        let content_op_size = [
            parent_size[0].map(|v| v - self.left - self.right),
            parent_size[1].map(|v| v - self.top - self.bottom),
        ];

        self.content
            .as_mut()
            .map(|content| {
                content
                    .cover_range(content_op_size, context)
                    .slide([self.left, self.top])
            })
            .unwrap_or_default()
    }

    fn need_rerendering(&self) -> bool {
        self.content
            .as_ref()
            .map(|content| content.need_rerendering())
            .unwrap_or(false)
    }

    fn render(
        &mut self,
        render_pass: &mut wgpu::RenderPass<'_>,
        target_size: [u32; 2],
        target_format: wgpu::TextureFormat,
        parent_size: [Option<f32>; 2],
        background: Background,
        ctx: &WidgetContext,
    ) -> Vec<matcha_core::ui::Object> {
        self.content
            .as_mut()
            .map(|content| {
                content.render(
                    render_pass,
                    target_size,
                    target_format,
                    parent_size,
                    background,
                    ctx,
                )
            })
            .unwrap_or_default()
            .into_iter()
            .map(|mut object| {
                object.transform(nalgebra::Matrix4::new_translation(&nalgebra::Vector3::new(
                    self.left, self.top, 0.0,
                )));
                object
            })
            .collect::<Vec<_>>()
    }
}
