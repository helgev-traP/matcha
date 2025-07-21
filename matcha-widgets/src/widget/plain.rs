use std::{any::Any, sync::Arc};

use matcha_core::{
    common_resource::CommonResource,
    context::WidgetContext,
    events::Event,
    observer::Observer,
    types::range::{CoverRange, Range2D},
    ui::{Background, Dom, DomComPareResult, Object, Style, UpdateWidgetError, Widget},
};

use crate::{
    buffer::Buffer,
    types::size::{ChildSize, Size},
};

// todo: more documentation

// MARK: DOM

#[derive(Default)]
pub struct Plain<T> {
    label: Option<String>,
    size: [Size; 2],
    style: Vec<Box<dyn Style>>,
    content: Option<Box<dyn Dom<T>>>,
}

impl<T> Plain<T> {
    pub fn new(label: Option<&str>) -> Box<Self> {
        Box::new(Self {
            label: label.map(|s| s.to_string()),
            size: [Size::parent(1.0), Size::parent(1.0)],
            style: Vec::new(),
            content: None,
        })
    }

    pub fn style(mut self, style: Box<dyn Style>) -> Self {
        self.style.push(style);
        self
    }
}

#[async_trait::async_trait]
impl<T: Send + 'static> Dom<T> for Plain<T> {
    fn build_widget_tree(&self) -> Box<dyn Widget<T>> {
        Box::new(PlainNode {
            label: self.label.clone(),
            size: self.size.clone(),
            buffer: Buffer::new(self.style.clone()),
            content: self
                .content
                .as_ref()
                .map(|content| content.build_widget_tree()),
            need_rerendering: true,
        })
    }

    async fn set_observer(&self) -> Observer {
        if let Some(content) = &self.content {
            content.set_observer().await
        } else {
            Observer::default()
        }
    }
}

// MARK: Widget

pub struct PlainNode<T> {
    label: Option<String>,
    size: [Size; 2],
    buffer: Buffer,
    content: Option<Box<dyn Widget<T>>>,

    need_rerendering: bool,
}

// MARK: Widget trait

#[async_trait::async_trait]
impl<T: Send + 'static> Widget<T> for PlainNode<T> {
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
        if let Some(dom) = (dom as &dyn Any).downcast_ref::<Plain<T>>() {
            todo!()
        } else {
            return Err(UpdateWidgetError::TypeMismatch);
        }
    }

    // comparing dom
    fn compare(&self, dom: &dyn Dom<T>) -> DomComPareResult {
        if let Some(dom) = (dom as &dyn Any).downcast_ref::<Plain<T>>() {
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
        ctx: &WidgetContext,
    ) -> Option<T> {
        let _ = (event, parent_size, ctx);
        todo!()
    }

    // inside / outside check
    // implement this if your widget has a non rectangular shape or has transparent area.
    fn is_inside(
        &mut self,
        position: [f32; 2],
        parent_size: [Option<f32>; 2],
        ctx: &WidgetContext,
    ) -> bool {
        let px_size = self.px_size(parent_size, ctx);

        if self.buffer.is_inside(position, px_size, ctx) {
            return true;
        }

        if let Some(content) = &mut self.content {
            content.is_inside(position, parent_size, ctx)
        } else {
            false
        }
    }

    // Actual size including its sub widgets with pixel value.
    fn px_size(&mut self, parent_size: [Option<f32>; 2], ctx: &WidgetContext) -> [f32; 2] {
        let _ = (parent_size, ctx);

        let child_size_f = || {
            if let Some(content) = &mut self.content {
                content.px_size(parent_size, ctx)
            } else {
                [0.0, 0.0]
            }
        };

        let mut child_size = ChildSize::new(child_size_f);

        let boundary_width = match &self.size[0] {
            Size::Size(f) => f(parent_size, &mut child_size, ctx),
            Size::Grow(_) => child_size.get()[0],
        };

        let boundary_height = match &self.size[1] {
            Size::Size(f) => f(parent_size, &mut child_size, ctx),
            Size::Grow(_) => child_size.get()[1],
        };

        [boundary_width, boundary_height]
    }

    // The drawing range and the area that the widget always covers.
    fn cover_range(
        &mut self,
        parent_size: [Option<f32>; 2],
        ctx: &WidgetContext,
    ) -> CoverRange<f32> {
        todo!()
    }

    // if redraw is needed
    fn need_rerendering(&self) -> bool {
        self.need_rerendering
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
        let px_size = self.px_size(parent_size, ctx);

        // render the buffer
        todo!()
    }
}
