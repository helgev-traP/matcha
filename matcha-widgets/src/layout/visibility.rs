use std::any::Any;

use matcha_core::{
    context::WidgetContext,
    device_event::DeviceEvent,
    observer::Observer,
    types::range::CoverRange,
    ui::{Background, Dom, DomComPareResult, Object, UpdateWidgetError, Widget},
};

#[derive(Debug, Clone, Copy)]
enum VisibilityState {
    Visible,
    Hidden,
    None,
}

pub struct Visibility<T>
where
    T: Send + 'static,
{
    // label
    label: Option<String>,

    // properties
    visible: VisibilityState,

    // content
    content: Option<Box<dyn Dom<T>>>,
}

impl<T> Default for Visibility<T>
where
    T: Send + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

// constructor
impl<T> Visibility<T>
where
    T: Send + 'static,
{
    pub fn new() -> Self {
        Self {
            label: None,
            visible: VisibilityState::Visible,
            content: None,
        }
    }

    pub fn label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    pub fn visible(mut self) -> Self {
        self.visible = VisibilityState::Visible;
        self
    }

    pub fn hidden(mut self) -> Self {
        self.visible = VisibilityState::Hidden;
        self
    }

    pub fn none(mut self) -> Self {
        self.visible = VisibilityState::None;
        self
    }

    pub fn content(mut self, content: Box<dyn Dom<T>>) -> Self {
        self.content = Some(content);
        self
    }
}

#[async_trait::async_trait]
impl<T> Dom<T> for Visibility<T>
where
    T: Send + 'static,
{
    fn build_widget_tree(&self) -> Box<dyn Widget<T>> {
        Box::new(VisibilityNode {
            label: self.label.clone(),
            visible: self.visible,
            content: self
                .content
                .as_ref()
                .map(|content| content.build_widget_tree()),
        })
    }

    async fn set_observer(&self) -> Observer {
        match self.visible {
            VisibilityState::Visible => {
                if let Some(content) = &self.content {
                    content.set_observer().await
                } else {
                    Observer::default()
                }
            }
            VisibilityState::Hidden | VisibilityState::None => Observer::default(),
        }
    }
}

pub struct VisibilityNode<T>
where
    T: Send + 'static,
{
    // label
    label: Option<String>,

    // properties
    visible: VisibilityState,

    // content
    content: Option<Box<dyn Widget<T>>>,
}

#[async_trait::async_trait]
impl<T> Widget<T> for VisibilityNode<T>
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
        if let Some(dom) = (dom as &dyn Any).downcast_ref::<Visibility<T>>() {
            // update properties
            self.label = dom.label.clone();

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
        if let Some(dom) = (dom as &dyn Any).downcast_ref::<Visibility<T>>() {
            let _ = dom;
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
        match self.visible {
            VisibilityState::Visible => self
                .content
                .as_mut()
                .map(|content| content.device_event(event, parent_size, context))
                .unwrap_or_default(),
            VisibilityState::Hidden | VisibilityState::None => None,
        }
    }

    fn px_size(&mut self, parent_size: [Option<f32>; 2], context: &WidgetContext) -> [f32; 2] {
        match self.visible {
            VisibilityState::Visible | VisibilityState::Hidden => self
                .content
                .as_mut()
                .map(|content| content.px_size(parent_size, context))
                .unwrap_or([0.0, 0.0]),
            VisibilityState::None => [0.0, 0.0],
        }
    }

    fn cover_range(
        &mut self,
        parent_size: [Option<f32>; 2],
        context: &WidgetContext,
    ) -> CoverRange<f32> {
        match self.visible {
            VisibilityState::Visible => self
                .content
                .as_mut()
                .map(|content| content.cover_range(parent_size, context))
                .unwrap_or_default(),
            VisibilityState::Hidden | VisibilityState::None => CoverRange::default(),
        }
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
    ) -> Vec<Object> {
        match self.visible {
            VisibilityState::Visible => self
                .content
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
                .unwrap_or_default(),
            VisibilityState::Hidden | VisibilityState::None => vec![],
        }
    }
}
