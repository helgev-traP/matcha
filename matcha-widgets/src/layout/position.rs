use std::any::Any;

use matcha_core::{
    common_resource::CommonResource,
    context::WidgetContext,
    events::Event,
    observer::Observer,
    types::range::{CoverRange, Range2D},
    ui::{Background, Dom, DomComPareResult, Object, UpdateWidgetError, Widget},
};

// MARK: DOM

pub struct Position {
    label: Option<String>,
}

#[async_trait::async_trait]
impl<T: Send + 'static> Dom<T> for Position {
    fn build_widget_tree(&self) -> Box<dyn Widget<T>> {
        Box::new(PositionNode {
            label: self.label.clone(),
        })
    }

    async fn collect_observer(&self) -> Observer {
        // If your widget has any child widgets,
        // you should collect their observers for matcha ui system to catch child component updates.

        Observer::default()
    }
}

// MARK: Widget

pub struct PositionNode {
    label: Option<String>,
}

// MARK: Widget trait

#[async_trait::async_trait]
impl<T: Send + 'static> Widget<T> for PositionNode {
    // label
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    // for dom handling
    // keep in mind to change redraw flag to true if some change is made.
    async fn update_widget_tree(
        &mut self,
        component_updated: bool,
        dom: &dyn Dom<T>,
    ) -> Result<(), UpdateWidgetError> {
        if let Some(dom) = (dom as &dyn Any).downcast_ref::<Position>() {
            todo!()
        } else {
            return Err(UpdateWidgetError::TypeMismatch);
        }
    }

    // comparing dom
    fn compare(&self, dom: &dyn Dom<T>) -> DomComPareResult {
        if let Some(dom) = (dom as &dyn Any).downcast_ref::<Position>() {
            todo!()
        } else {
            DomComPareResult::Different
        }
    }

    // widget event
    fn widget_event(
        &mut self,
        event: &Event,
        parent_size: [Option<f32>; 2],
        context: &WidgetContext,
    ) -> Option<T> {
        let _ = (event, parent_size, context);
        todo!()
    }

    // inside / outside check
    // implement this if your widget has a non rectangular shape.
    /*
    fn is_inside(
        &mut self,
        position: [f32; 2],
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
    ) -> bool {
        let px_size = Widget::<T>::px_size(self, parent_size, context);

        !(position[0] < 0.0
            || position[0] > px_size[0]
            || position[1] < 0.0
            || position[1] > px_size[1])
    }
    */

    // Actual size including its sub widgets with pixel value.
    fn px_size(&mut self, parent_size: [Option<f32>; 2], context: &WidgetContext) -> [f32; 2] {
        let _ = (parent_size, context);
        todo!()
    }

    // The drawing range and the area that the widget always covers.
    fn cover_range(
        &mut self,
        parent_size: [Option<f32>; 2],
        context: &WidgetContext,
    ) -> CoverRange<f32> {
        todo!()
    }

    // if redraw is needed
    fn need_rerendering(&self) -> bool {
        todo!()
    }

    // render
    fn render(
        &mut self,
        render_pass: &mut wgpu::RenderPass<'_>,
        target_size: [u32; 2],
        target_format: wgpu::TextureFormat,
        parent_size: [Option<f32>; 2],
        background: Background,
        ctx: &WidgetContext,
    ) -> Vec<Object> {
        todo!()
    }
}
