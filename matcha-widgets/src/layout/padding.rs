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

pub struct Padding<T>
where
    T: Send + 'static,
{
    label: Option<String>,
    top: f32,
    right: f32,
    bottom: f32,
    left: f32,
    content: Option<Box<dyn Dom<T>>>,
}

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

    async fn set_update_notifier(&self, notifier: &UpdateNotifier) {
        if let Some(content) = &self.content {
            content.set_update_notifier(notifier).await;
        }
    }
}

pub struct PaddingNode<T>
where
    T: Send + 'static,
{
    label: Option<String>,
    top: f32,
    right: f32,
    bottom: f32,
    left: f32,
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
        _component_updated: bool,
        dom: &dyn Dom<T>,
    ) -> Result<(), UpdateWidgetError> {
        if let Some(dom) = (dom as &dyn Any).downcast_ref::<Padding<T>>() {
            self.top = dom.top;
            self.right = dom.right;
            self.bottom = dom.bottom;
            self.left = dom.left;
            // Simplified content update
            Ok(())
        } else {
            Err(UpdateWidgetError::TypeMismatch)
        }
    }

    fn compare(&self, dom: &dyn Dom<T>) -> DomCompareResult {
        if (dom as &dyn Any).downcast_ref::<Padding<T>>().is_some() {
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
        let inner_pos = [position[0] - self.left, position[1] - self.top];
        self.content
            .as_mut()
            .map_or(false, |c| c.is_inside(inner_pos, context))
    }

    fn preferred_size(&mut self, constraints: &Constraints, context: &WidgetContext) -> [f32; 2] {
        let content_size = self.content.as_mut().map_or([0.0, 0.0], |c| {
            let inner_constraints = Constraints {
                min_width: (constraints.min_width - self.left - self.right).max(0.0),
                max_width: (constraints.max_width - self.left - self.right).max(0.0),
                min_height: (constraints.min_height - self.top - self.bottom).max(0.0),
                max_height: (constraints.max_height - self.top - self.bottom).max(0.0),
            };
            c.preferred_size(&inner_constraints, context)
        });

        [
            content_size[0] + self.left + self.right,
            content_size[1] + self.top + self.bottom,
        ]
    }

    fn arrange(&mut self, final_size: [f32; 2], context: &WidgetContext) {
        if let Some(content) = &mut self.content {
            let content_size = [
                (final_size[0] - self.left - self.right).max(0.0),
                (final_size[1] - self.top - self.bottom).max(0.0),
            ];
            content.arrange(content_size, context);
        }
    }

    fn need_rerendering(&self) -> bool {
        self.content
            .as_ref()
            .map_or(false, |c| c.need_rerendering())
    }

    fn render(&mut self, background: Background, ctx: &WidgetContext) -> RenderNode {
        if let Some(content) = &mut self.content {
            let transform = nalgebra::Matrix4::new_translation(&nalgebra::Vector3::new(
                self.left, self.top, 0.0,
            ));
            let child_node = content.render(background.translate([self.left, self.top]), ctx);
            let mut render_node = RenderNode::new();
            render_node.add_child(child_node, transform);
            render_node
        } else {
            RenderNode::new()
        }
    }
}
