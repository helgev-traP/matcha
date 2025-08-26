use std::any::Any;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use matcha_core::{
    device_event::DeviceEvent,
    render_node::RenderNode,
    types::range::{CoverRange, Range2D},
    ui::{
        Background, Constraints, Dom, DomCompareResult, Style, UpdateWidgetError, Widget,
        WidgetContext,
    },
    update_flag::UpdateNotifier,
};
use nalgebra::constraint;
use texture_atlas::atlas_simple::atlas::AtlasRegion;
use utils::cache::Cache;

use crate::types::size::{ChildSize, Size};

// todo: more documentation

// MARK: DOM

pub struct Plain<T> {
    label: Option<String>,
    style: Vec<Box<dyn Style>>,
    content: Option<Box<dyn Dom<T>>>,

    size: [Size; 2],
}

impl<T> Plain<T> {
    pub fn new(label: Option<&str>) -> Box<Self> {
        Box::new(Self {
            label: label.map(|s| s.to_string()),
            style: Vec::new(),
            content: None,
            size: [Size::child_w(1.0), Size::child_h(1.0)],
        })
    }

    pub fn style(mut self, style: Box<dyn Style>) -> Self {
        self.style.push(style);
        self
    }

    pub fn content(mut self, content: Box<dyn Dom<T>>) -> Self {
        self.content = Some(content);
        self
    }

    pub fn size(mut self, size: [Size; 2]) -> Self {
        self.size = size;
        self
    }
}

#[async_trait::async_trait]
impl<T: Send + 'static> Dom<T> for Plain<T> {
    fn build_widget_tree(&self) -> Box<dyn Widget<T>> {
        Box::new(PlainNode {
            label: self.label.clone(),
            style: self.style.clone(),
            content: self
                .content
                .as_ref()
                .map(|content| content.build_widget_tree()),
            size: self.size.clone(),
            need_rerendering: true,
            cache: Cache::new(),
        })
    }

    async fn set_update_notifier(&self, notifier: &UpdateNotifier) {
        if let Some(content) = &self.content {
            content.set_update_notifier(notifier).await;
        }
    }
}

// MARK: Widget

pub struct PlainNode<T> {
    label: Option<String>,
    style: Vec<Box<dyn Style>>,
    content: Option<Box<dyn Widget<T>>>,
    size: [Size; 2],

    // system
    need_rerendering: bool,
    cache: Cache<[Option<f32>; 2], PlainNodeCache>,
}

struct PlainNodeCache {
    arranged_size: [f32; 2],
    region: Option<AtlasRegion>,
}

// MARK: Widget trait

#[async_trait::async_trait]
impl<T: Send + 'static> Widget<T> for PlainNode<T> {
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    async fn update_widget_tree(
        &mut self,
        _component_updated: bool,
        dom: &dyn Dom<T>,
    ) -> Result<(), UpdateWidgetError> {
        if let Some(dom) = (dom as &dyn Any).downcast_ref::<Plain<T>>() {
            self.label = dom.label.clone();
            self.style = dom.style.clone();
            // Proper content update logic is needed here.
            // This might involve comparing and updating the child widget.
            Ok(())
        } else {
            Err(UpdateWidgetError::TypeMismatch)
        }
    }

    fn compare(&self, dom: &dyn Dom<T>) -> DomCompareResult {
        if (dom as &dyn Any).downcast_ref::<Plain<T>>().is_some() {
            DomCompareResult::Same // Simplified for now
        } else {
            DomCompareResult::Different
        }
    }

    fn device_event(&mut self, event: &DeviceEvent, context: &WidgetContext) -> Option<T> {
        if let Some(content) = &mut self.content {
            return content.device_event(event, context);
        }
        None
    }

    fn is_inside(&mut self, position: [f32; 2], context: &WidgetContext) -> bool {
        self.cache
            .get()
            .map(|cache| {
                let size = cache.1.arranged_size;
                0.0 <= position[0]
                    && position[0] <= size[0]
                    && 0.0 <= position[1]
                    && position[1] <= size[1]
            })
            .unwrap_or(false)
    }

    fn preferred_size(&mut self, constraints: &Constraints, context: &WidgetContext) -> [f32; 2] {
        let child_constraints_width = self.size[0].constraints(constraints, context);
        let child_constraints_height = self.size[1].constraints(constraints, context);
        let child_constraints = Constraints::new(child_constraints_width, child_constraints_height);

        todo!()
    }

    fn arrange(&mut self, final_size: [f32; 2], context: &WidgetContext) {
        todo!()
    }

    fn need_rerendering(&self) -> bool {
        true
    }

    fn render(&mut self, background: Background, ctx: &WidgetContext) -> RenderNode {
        todo!()
    }
}
