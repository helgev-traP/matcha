use nalgebra::Matrix4;

use matcha_core::ui::widget::InvalidationHandle;
use matcha_core::{
    device_input::DeviceInput,
    ui::{
        AnyWidget, AnyWidgetFrame, Arrangement, Background, Constraints, Dom, Widget,
        WidgetContext, WidgetFrame,
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

    pub fn left(mut self, left: f32) -> Self {
        self.left = Some(left);
        self
    }

    pub fn top(mut self, top: f32) -> Self {
        self.top = Some(top);
        self
    }

    pub fn right(mut self, right: f32) -> Self {
        self.right = Some(right);
        self
    }

    pub fn bottom(mut self, bottom: f32) -> Self {
        self.bottom = Some(bottom);
        self
    }

    pub fn content(mut self, content: Box<dyn Dom<T>>) -> Self {
        self.content = Some(content);
        self
    }
}

#[async_trait::async_trait]
impl<T: Send + 'static> Dom<T> for Position<T> {
    fn build_widget_tree(&self) -> Box<dyn AnyWidgetFrame<T>> {
        Box::new(WidgetFrame::new(
            self.label.clone(),
            vec![],
            vec![],
            PositionNode {
                left: self.left,
                top: self.top,
                right: self.right,
                bottom: self.bottom,
            },
        ))
    }

    async fn set_update_notifier(&self, notifier: &UpdateNotifier) {
        if let Some(content) = &self.content {
            content.set_update_notifier(notifier).await;
        }
    }
}

// MARK: Widget

pub struct PositionNode {
    left: Option<f32>,
    top: Option<f32>,
    right: Option<f32>,
    bottom: Option<f32>,
}

impl<T: Send + 'static> Widget<Position<T>, T, ()> for PositionNode {
    fn update_widget<'a>(
        &mut self,
        dom: &'a Position<T>,
        _cache_invalidator: Option<InvalidationHandle>,
    ) -> Vec<(&'a dyn Dom<T>, (), u128)> {
        self.left = dom.left;
        self.top = dom.top;
        self.right = dom.right;
        self.bottom = dom.bottom;
        dom.content
            .as_ref()
            .map(|c| (c.as_ref(), (), 0))
            .into_iter()
            .collect()
    }

    fn device_event(
        &mut self,
        _bounds: [f32; 2],
        event: &DeviceInput,
        children: &mut [(&mut dyn AnyWidget<T>, &mut (), &Arrangement)],
        _cache_invalidator: InvalidationHandle,
        ctx: &WidgetContext,
    ) -> Option<T> {
        if let Some((child, _, _arrangement)) = children.first_mut() {
            return child.device_event(event, ctx);
        }
        None
    }

    fn is_inside(
        &self,
        _bounds: [f32; 2],
        position: [f32; 2],
        children: &[(&dyn AnyWidget<T>, &(), &Arrangement)],
        ctx: &WidgetContext,
    ) -> bool {
        if let Some((child, _, _arrangement)) = children.first() {
            return child.is_inside(position, ctx);
        }
        false
    }

    fn measure(
        &self,
        constraints: &Constraints,
        children: &[(&dyn AnyWidget<T>, &())],
        ctx: &WidgetContext,
    ) -> [f32; 2] {
        if let Some((child, _)) = children.first() {
            child.measure(constraints, ctx)
        } else {
            [0.0, 0.0]
        }
    }

    fn arrange(
        &self,
        size: [f32; 2],
        children: &[(&dyn AnyWidget<T>, &())],
        ctx: &WidgetContext,
    ) -> Vec<Arrangement> {
        if children.is_empty() {
            return vec![];
        }

        let child_measured_size = if let Some((child, _)) = children.first() {
            child.measure(&Constraints::new([0.0, 0.0], size), ctx)
        } else {
            [0.0, 0.0]
        };

        let x = self
            .left
            .unwrap_or_else(|| size[0] - child_measured_size[0] - self.right.unwrap_or(0.0));
        let y = self
            .top
            .unwrap_or_else(|| size[1] - child_measured_size[1] - self.bottom.unwrap_or(0.0));

        let w = if let (Some(left), Some(right)) = (self.left, self.right) {
            (size[0] - left - right).max(0.0)
        } else {
            child_measured_size[0]
        };
        let h = if let (Some(top), Some(bottom)) = (self.top, self.bottom) {
            (size[1] - top - bottom).max(0.0)
        } else {
            child_measured_size[1]
        };

        let content_size = [w, h];
        let transform = Matrix4::new_translation(&nalgebra::Vector3::new(x, y, 0.0));

        vec![Arrangement::new(content_size, transform)]
    }

    fn render(
        &self,
        background: Background,
        children: &[(&dyn AnyWidget<T>, &(), &Arrangement)],
        ctx: &WidgetContext,
    ) -> RenderNode {
        if let Some((child, _, arrangement)) = children.first() {
            let final_size = arrangement.size;
            let affine = arrangement.affine;

            let child_node = child.render(final_size, background, ctx);

            return RenderNode::new().add_child(child_node, affine);
        }
        RenderNode::default()
    }
}
