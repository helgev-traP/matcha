use std::any::Any;

use matcha_core::{
    device_event::DeviceEvent,
    render_node::RenderNode,
    types::range::CoverRange,
    ui::{
        Background, Constraints, Dom, DomCompareResult, UpdateWidgetError, Widget, WidgetContext,
    },
    update_flag::UpdateNotifier,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisibilityState {
    Visible,
    Hidden,
    Gone,
}

pub struct Visibility<T>
where
    T: Send + 'static,
{
    label: Option<String>,
    visibility: VisibilityState,
    content: Option<Box<dyn Dom<T>>>,
}

impl<T> Visibility<T>
where
    T: Send + 'static,
{
    pub fn new() -> Self {
        Self {
            label: None,
            visibility: VisibilityState::Visible,
            content: None,
        }
    }

    pub fn label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.visibility = if visible {
            VisibilityState::Visible
        } else {
            VisibilityState::Hidden
        };
        self
    }

    pub fn gone(mut self, gone: bool) -> Self {
        if gone {
            self.visibility = VisibilityState::Gone;
        }
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
            visibility: self.visibility,
            content: self
                .content
                .as_ref()
                .map(|content| content.build_widget_tree()),
        })
    }

    async fn set_update_notifier(&self, notifier: &UpdateNotifier) {
        if let Some(content) = &self.content {
            content.set_update_notifier(notifier).await;
        }
    }
}

pub struct VisibilityNode<T>
where
    T: Send + 'static,
{
    label: Option<String>,
    visibility: VisibilityState,
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
            self.label = dom.label.clone();
            self.visibility = dom.visibility;

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

    fn compare(&self, dom: &dyn Dom<T>) -> DomCompareResult {
        if (dom as &dyn Any).downcast_ref::<Visibility<T>>().is_some() {
            DomCompareResult::Same // Simplified
        } else {
            DomCompareResult::Different
        }
    }

    fn device_event(&mut self, event: &DeviceEvent, context: &WidgetContext) -> Option<T> {
        if self.visibility == VisibilityState::Visible {
            self.content
                .as_mut()
                .and_then(|content| content.device_event(event, context))
        } else {
            None
        }
    }

    fn is_inside(&mut self, position: [f32; 2], context: &WidgetContext) -> bool {
        if self.visibility == VisibilityState::Visible {
            self.content
                .as_mut()
                .map_or(false, |content| content.is_inside(position, context))
        } else {
            false
        }
    }

    fn preferred_size(&mut self, constraints: &Constraints, context: &WidgetContext) -> [f32; 2] {
        if self.visibility == VisibilityState::Gone {
            return [0.0, 0.0];
        }
        self.content
            .as_mut()
            .map_or([0.0, 0.0], |c| c.preferred_size(constraints, context))
    }

    fn arrange(&mut self, final_size: [f32; 2], context: &WidgetContext) {
        if let Some(content) = &mut self.content {
            content.arrange(final_size, context);
        }
    }

    fn need_rerendering(&self) -> bool {
        self.content
            .as_ref()
            .map_or(false, |content| content.need_rerendering())
    }

    fn render(&mut self, background: Background, ctx: &WidgetContext) -> RenderNode {
        if self.visibility == VisibilityState::Visible {
            if let Some(content) = &mut self.content {
                return content.render(background, ctx);
            }
        }

        RenderNode::new()
    }
}
