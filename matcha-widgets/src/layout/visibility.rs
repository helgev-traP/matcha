use std::any::Any;

use nalgebra::Matrix4;

use matcha_core::ui::widget::InvalidationHandle;
use matcha_core::{
    device_event::DeviceEvent,
    ui::{
        AnyWidget, AnyWidgetFrame, Arrangement, Background, Constraints, Dom, Widget,
        WidgetContext, WidgetFrame,
    },
    update_flag::UpdateNotifier,
};
use renderer::render_node::RenderNode;
use vello::low_level::Render;

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
    fn build_widget_tree(&self) -> Box<dyn AnyWidgetFrame<T>> {
        Box::new(WidgetFrame::new(
            self.label.clone(),
            vec![],
            vec![],
            VisibilityNode {
                visibility: self.visibility,
            },
        ))
    }

    async fn set_update_notifier(&self, notifier: &UpdateNotifier) {
        if let Some(content) = &self.content {
            content.set_update_notifier(notifier).await;
        }
    }
}

pub struct VisibilityNode {
    visibility: VisibilityState,
}

impl VisibilityNode {
    fn identity_affine() -> Matrix4<f32> {
        Matrix4::identity()
    }
}

impl<T> Widget<Visibility<T>, T, ()> for VisibilityNode
where
    T: Send + 'static,
{
    fn update_widget<'a>(
        &mut self,
        dom: &'a Visibility<T>,
        _cache_invalidator: Option<InvalidationHandle>,
    ) -> Vec<(&'a dyn Dom<T>, (), u128)> {
        self.visibility = dom.visibility;
        dom.content
            .as_ref()
            .map(|c| (c.as_ref(), (), 0))
            .into_iter()
            .collect()
    }

    fn device_event(
        &mut self,
        _bounds: [f32; 2],
        event: &DeviceEvent,
        children: &mut [(&mut dyn AnyWidget<T>, &mut (), &Arrangement)],
        _cache_invalidator: InvalidationHandle,
        ctx: &WidgetContext,
    ) -> Option<T> {
        if self.visibility == VisibilityState::Visible {
            if let Some((child, _, _arrangement)) = children.first() {
                return (*child).device_event(event, ctx);
            }
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
        if self.visibility == VisibilityState::Visible {
            if let Some((child, _, _arrangement)) = children.first() {
                return child.is_inside(position, ctx);
            }
        }
        false
    }

    fn measure(
        &self,
        constraints: &Constraints,
        children: &[(&dyn AnyWidget<T>, &())],
        ctx: &WidgetContext,
    ) -> [f32; 2] {
        if self.visibility == VisibilityState::Gone {
            return [0.0, 0.0];
        }

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
        _ctx: &WidgetContext,
    ) -> Vec<Arrangement> {
        if children.is_empty() {
            vec![]
        } else {
            vec![Arrangement::new(size, Self::identity_affine())]
        }
    }

    fn render(
        &self,
        background: Background,
        children: &[(&dyn AnyWidget<T>, &(), &Arrangement)],
        ctx: &WidgetContext,
    ) -> RenderNode {
        if self.visibility == VisibilityState::Visible {
            if let Some((child, _, arrangement)) = children.first() {
                let final_size = arrangement.size;
                let affine = arrangement.affine;

                let child_node = child.render(final_size, background, ctx);

                return RenderNode::new().add_child(child_node, affine);
            }
        }
        RenderNode::default()
    }
}
