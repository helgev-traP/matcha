use std::any::Any;

use matcha_core::{
    device_event::DeviceEvent,
    types::range::CoverRange,
    ui::{
        Background, Constraints, Dom, DomCompareResult, UpdateWidgetError, Widget, WidgetContext,
    },
    update_flag::UpdateNotifier,
};
use renderer::render_node::RenderNode;

// MARK: DOM

pub struct Position<T: Send + 'static> {
    label: Option<String>,
    left: Option<f32>,
    top: Option<f32>,
    right: Option<f32>,
    bottom: Option<f32>,
    content: Option<Box<dyn Dom<T>>>,
}

impl<T: Send + 'static> Position<T> {
    pub fn new() -> Self {
        Self {
            label: None,
            left: None,
            top: None,
            right: None,
            bottom: None,
            content: None,
        }
    }

    pub fn content(mut self, content: Box<dyn Dom<T>>) -> Self {
        self.content = Some(content);
        self
    }
}

#[async_trait::async_trait]
impl<T: Send + 'static> Dom<T> for Position<T> {
    fn build_widget_tree(&self) -> Box<dyn Widget<T>> {
        Box::new(PositionNode {
            label: self.label.clone(),
            left: self.left,
            top: self.top,
            right: self.right,
            bottom: self.bottom,
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

// MARK: Widget

pub struct PositionNode<T: Send + 'static> {
    label: Option<String>,
    left: Option<f32>,
    top: Option<f32>,
    right: Option<f32>,
    bottom: Option<f32>,
    content: Option<Box<dyn Widget<T>>>,
}

#[async_trait::async_trait]
impl<T: Send + 'static> Widget<T> for PositionNode<T> {
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    async fn update_widget_tree(
        &mut self,
        _component_updated: bool,
        dom: &dyn Dom<T>,
    ) -> Result<(), UpdateWidgetError> {
        if let Some(dom) = (dom as &dyn Any).downcast_ref::<Position<T>>() {
            self.left = dom.left;
            self.top = dom.top;
            self.right = dom.right;
            self.bottom = dom.bottom;
            // Simplified content update
            Ok(())
        } else {
            Err(UpdateWidgetError::TypeMismatch)
        }
    }

    fn compare(&self, dom: &dyn Dom<T>) -> DomCompareResult {
        if (dom as &dyn Any).downcast_ref::<Position<T>>().is_some() {
            DomCompareResult::Same // Simplified
        } else {
            DomCompareResult::Different
        }
    }

    fn device_event(&mut self, event: &DeviceEvent, context: &WidgetContext) -> Option<T> {
        self.content
            .as_mut()
            .and_then(|c| c.device_event(event, context))
    }

    fn is_inside(&mut self, position: [f32; 2], context: &WidgetContext) -> bool {
        // This needs to be adjusted based on the final position from arrange pass
        self.content
            .as_mut()
            .map_or(false, |c| c.is_inside(position, context))
    }

    fn preferred_size(&mut self, constraints: &Constraints, context: &WidgetContext) -> [f32; 2] {
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
            .map_or(false, |c| c.need_rerendering())
    }

    fn render(&mut self, background: Background, ctx: &WidgetContext) -> RenderNode {
        if let Some(content) = &mut self.content {
            let x = self.left.unwrap_or(0.0);
            let y = self.top.unwrap_or(0.0);
            // A full implementation would also handle right and bottom.

            let transform = nalgebra::Matrix4::new_translation(&nalgebra::Vector3::new(x, y, 0.0));
            let child_node = content.render(background.translate([x, y]), ctx);
            let mut render_node = RenderNode::new();
            render_node.add_child(child_node, transform);
            render_node
        } else {
            RenderNode::new()
        }
    }
}
