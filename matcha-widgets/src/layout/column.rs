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

use crate::types::flex::{AlignItems, JustifyContent};

// MARK: DOM

pub struct Column<T> {
    pub label: Option<String>,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub items: Vec<Box<dyn Dom<T>>>,
}

impl<T> Column<T> {
    pub fn new(label: Option<&str>) -> Box<Self> {
        Box::new(Self {
            label: label.map(String::from),
            justify_content: JustifyContent::FlexStart {
                gap: std::sync::Arc::new(|_, _| 0.0),
            },
            align_items: AlignItems::Start,
            items: Vec::new(),
        })
    }

    pub fn push(mut self, item: Box<dyn Dom<T>>) -> Self {
        self.items.push(item);
        self
    }
}

#[async_trait::async_trait]
impl<T: Send + 'static> Dom<T> for Column<T> {
    fn build_widget_tree(&self) -> Box<dyn Widget<T>> {
        Box::new(ColumnNode {
            label: self.label.clone(),
            justify_content: self.justify_content.clone(),
            align_items: self.align_items,
            items: self
                .items
                .iter()
                .map(|item| item.build_widget_tree())
                .collect(),
            item_sizes: Vec::new(),
            size: [0.0, 0.0],
        })
    }

    async fn set_update_notifier(&self, notifier: &UpdateNotifier) {
        for item in &self.items {
            item.set_update_notifier(notifier).await;
        }
    }
}

// MARK: Widget

pub struct ColumnNode<T> {
    label: Option<String>,
    justify_content: JustifyContent,
    align_items: AlignItems,
    items: Vec<Box<dyn Widget<T>>>,
    item_sizes: Vec<[f32; 2]>,
    size: [f32; 2],
}

#[async_trait::async_trait]
impl<T: Send + 'static> Widget<T> for ColumnNode<T> {
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    async fn update_widget_tree(
        &mut self,
        _component_updated: bool,
        dom: &dyn Dom<T>,
    ) -> Result<(), UpdateWidgetError> {
        if let Some(dom) = (dom as &dyn Any).downcast_ref::<Column<T>>() {
            self.label = dom.label.clone();
            self.justify_content = dom.justify_content.clone();
            self.align_items = dom.align_items;
            // This is a simplified update. A real implementation would diff the items.
            self.items = dom
                .items
                .iter()
                .map(|item| item.build_widget_tree())
                .collect();
            Ok(())
        } else {
            Err(UpdateWidgetError::TypeMismatch)
        }
    }

    fn compare(&self, dom: &dyn Dom<T>) -> DomComPareResult {
        if (dom as &dyn Any).downcast_ref::<Column<T>>().is_some() {
            DomComPareResult::Same // Simplified
        } else {
            DomComPareResult::Different
        }
    }

    fn device_event(&mut self, event: &DeviceEvent, context: &WidgetContext) -> Option<T> {
        self.items
            .iter_mut()
            .find_map(|item| item.device_event(event, context))
    }

    fn is_inside(&mut self, position: [f32; 2], context: &WidgetContext) -> bool {
        self.items
            .iter_mut()
            .any(|item| item.is_inside(position, context))
    }

    fn preferred_size(&self, constraints: &Constraints, context: &WidgetContext) -> [f32; 2] {
        let mut total_height = 0.0;
        let mut max_width: f32 = 0.0;
        self.item_sizes.clear();

        for item in &mut self.items {
            let item_size = item.preferred_size(constraints, context);
            self.item_sizes.push(item_size);
            total_height += item_size[1];
            max_width = max_width.max(item_size[0]);
        }

        [max_width, total_height]
    }

    fn arrange(&mut self, final_size: [f32; 2], context: &WidgetContext) {
        self.size = final_size;
        let mut y_pos = 0.0;
        for (item, &item_size) in self.items.iter_mut().zip(&self.item_sizes) {
            // This is a simplified arrangement (FlexStart).
            // A full implementation would handle justify_content and align_items.
            let child_final_size = [final_size[0], item_size[1]];
            item.arrange(child_final_size, context);
            y_pos += item_size[1];
        }
    }

    fn need_rerendering(&self) -> bool {
        self.items.iter().any(|item| item.need_rerendering())
    }

    fn render(&mut self, background: Background, ctx: &WidgetContext) -> RenderNode {
        let mut render_node = RenderNode::new();
        let mut y_pos = 0.0;

        for (item, &item_size) in self.items.iter_mut().zip(&self.item_sizes) {
            let transform =
                nalgebra::Matrix4::new_translation(&nalgebra::Vector3::new(0.0, y_pos, 0.0));
            let child_node = item.render(background.transition([0.0, y_pos]), ctx);
            render_node.add_child(child_node, transform);
            y_pos += item_size[1];
        }

        render_node
    }
}
