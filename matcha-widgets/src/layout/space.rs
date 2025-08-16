use std::any::Any;
use std::sync::Arc;

use matcha_core::{
    device_event::DeviceEvent,
    render_node::RenderNode,
    types::range::CoverRange,
    ui::{
        Background, Constraints, Dom, DomComPareResult, UpdateWidgetError, Widget, WidgetContext,
    },
    update_flag::UpdateNotifier,
};

// todo: more documentation

// MARK: DOM

type SizeFn = dyn for<'a> Fn(&Constraints, &'a WidgetContext) -> [f32; 2] + Send + Sync + 'static;

pub struct Space {
    label: Option<String>,
    size: Arc<SizeFn>,
}

impl Space {
    pub fn new(label: Option<&str>) -> Box<Self> {
        Box::new(Self {
            label: label.map(|s| s.to_string()),
            size: Arc::new(|constraints, _| [constraints.min_width, constraints.min_height]),
        })
    }

    pub fn size<F>(mut self, size: F) -> Self
    where
        F: Fn(&Constraints, &WidgetContext) -> [f32; 2] + Send + Sync + 'static,
    {
        self.size = Arc::new(size);
        self
    }
}

#[async_trait::async_trait]
impl<T: Send + 'static> Dom<T> for Space {
    fn build_widget_tree(&self) -> Box<dyn Widget<T>> {
        Box::new(SpaceNode {
            label: self.label.clone(),
            size: Arc::clone(&self.size),
            final_size: [0.0, 0.0],
        })
    }

    async fn set_update_notifier(&self, _notifier: &UpdateNotifier) {
        // No children to notify
    }
}

// MARK: Widget

pub struct SpaceNode {
    label: Option<String>,
    size: Arc<SizeFn>,
    final_size: [f32; 2],
}

// MARK: Widget trait

#[async_trait::async_trait]
impl<T: Send + 'static> Widget<T> for SpaceNode {
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    async fn update_widget_tree(
        &mut self,
        _: bool,
        dom: &dyn Dom<T>,
    ) -> Result<(), UpdateWidgetError> {
        if let Some(dom) = (dom as &dyn Any).downcast_ref::<Space>() {
            self.label = dom.label.clone();
            self.size = Arc::clone(&dom.size);
            Ok(())
        } else {
            Err(UpdateWidgetError::TypeMismatch)
        }
    }

    fn compare(&self, dom: &dyn Dom<T>) -> DomComPareResult {
        if (dom as &dyn Any).downcast_ref::<Space>().is_some() {
            DomComPareResult::Same // Simplified
        } else {
            DomComPareResult::Different
        }
    }

    fn device_event(&mut self, _event: &DeviceEvent, _context: &WidgetContext) -> Option<T> {
        None
    }

    fn is_inside(&mut self, _position: [f32; 2], _context: &WidgetContext) -> bool {
        // A space is just an empty area, so it's never "inside".
        false
    }

    fn preferred_size(&mut self, constraints: &Constraints, context: &WidgetContext) -> [f32; 2] {
        (self.size)(constraints, context)
    }

    fn arrange(&mut self, final_size: [f32; 2], _context: &WidgetContext) {
        self.final_size = final_size;
    }

    fn cover_range(&mut self, _context: &WidgetContext) -> CoverRange<f32> {
        CoverRange::default()
    }

    fn need_rerendering(&self) -> bool {
        false
    }

    fn render(
        &mut self,
        _background: Background,
        _animation_update_flag_notifier: UpdateNotifier,
        _ctx: &WidgetContext,
    ) -> RenderNode {
        RenderNode::new()
    }
}
