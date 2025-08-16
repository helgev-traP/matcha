use std::any::Any;

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

pub struct Template {
    label: Option<String>,
}

impl Template {
    pub fn new(label: Option<&str>) -> Box<Self> {
        Box::new(Self {
            label: label.map(|s| s.to_string()),
        })
    }
}

#[async_trait::async_trait]
impl<T: Send + 'static> Dom<T> for Template {
    fn build_widget_tree(&self) -> Box<dyn Widget<T>> {
        Box::new(TemplateNode {
            label: self.label.clone(),
            update_notifier: None,
        })
    }

    async fn set_update_notifier(&self, _notifier: &UpdateNotifier) {
        // If your widget has any child widgets,
        // you should propagate the notifier to them.
    }
}

// MARK: Widget

pub struct TemplateNode {
    label: Option<String>,
    update_notifier: Option<UpdateNotifier>,
}

// MARK: Widget trait

#[async_trait::async_trait]
impl<T: Send + 'static> Widget<T> for TemplateNode {
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    async fn update_widget_tree(
        &mut self,
        _component_updated: bool,
        dom: &dyn Dom<T>,
    ) -> Result<(), UpdateWidgetError> {
        if let Some(dom) = (dom as &dyn Any).downcast_ref::<Template>() {
            self.label = dom.label.clone();
            Ok(())
        } else {
            Err(UpdateWidgetError::TypeMismatch)
        }
    }

    fn compare(&self, dom: &dyn Dom<T>) -> DomComPareResult {
        if (dom as &dyn Any).downcast_ref::<Template>().is_some() {
            // In a real widget, you would compare properties here.
            // If properties are different, return DomComPareResult::Changed(hash).
            DomComPareResult::Same
        } else {
            DomComPareResult::Different
        }
    }

    fn device_event(&mut self, _event: &DeviceEvent, _context: &WidgetContext) -> Option<T> {
        // Handle device events here.
        // If the widget's state changes and it needs to be redrawn,
        // call self.update_notifier.as_ref().unwrap().notify();
        None
    }

    fn is_inside(&mut self, _position: [f32; 2], _context: &WidgetContext) -> bool {
        // Implement this if your widget has a non-rectangular shape or transparent areas.
        // For a simple template, we can assume it's always inside its bounds.
        true
    }

    fn preferred_size(&mut self, _constraints: &Constraints, _context: &WidgetContext) -> [f32; 2] {
        // This widget has no content, so it takes up no space.
        [0.0, 0.0]
    }

    fn arrange(&mut self, _final_size: [f32; 2], _context: &WidgetContext) {
        // This widget has no children to arrange.
    }

    fn cover_range(&mut self, _context: &WidgetContext) -> CoverRange<f32> {
        // This widget is transparent and covers no area.
        CoverRange::default()
    }

    fn need_rerendering(&self) -> bool {
        // A real widget would have state to track this.
        // For a template, we can assume it doesn't need rerendering unless updated.
        false
    }

    fn render(
        &mut self,
        _background: Background,
        animation_update_flag_notifier: UpdateNotifier,
        _ctx: &WidgetContext,
    ) -> RenderNode {
        // Store the notifier so we can request a redraw later.
        self.update_notifier = Some(animation_update_flag_notifier);

        // This widget doesn't draw anything.
        RenderNode::new()
    }
}
