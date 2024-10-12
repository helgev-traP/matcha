use nalgebra as na;
use std::{any::Any, cell::Cell};

use super::{Dom, DomComPareResult, RenderingTrait, Widget, WidgetTrait};
use crate::{
    application_context::ApplicationContext,
    events::UiEventResult,
    renderer::RendererCommandEncoder,
    types::size::{PxSize, Size, SizeUnit, StdSize, StdSizeUnit},
};

pub struct ColumnDescriptor<R> {
    pub label: Option<String>,
    pub children: Vec<Box<dyn Dom<R>>>,
}

impl<R> Default for ColumnDescriptor<R> {
    fn default() -> Self {
        Self {
            label: None,
            children: Vec::new(),
        }
    }
}

pub struct Column<R: 'static> {
    label: Option<String>,
    children: Vec<Box<dyn Dom<R>>>,
}

impl<R: 'static> Column<R> {
    pub fn new(disc: ColumnDescriptor<R>) -> Self {
        Self {
            label: disc.label,
            children: disc.children,
        }
    }

    pub fn push(&mut self, child: Box<dyn Dom<R>>) {
        self.children.push(child);
    }
}

impl<R: 'static> Dom<R> for Column<R> {
    fn build_render_tree(&self) -> Box<dyn Widget<R>> {
        let mut render_tree = Vec::new();

        for child in &self.children {
            render_tree.push(child.build_render_tree());
        }

        Box::new(ColumnRenderNode {
            label: self.label.clone(),
            redraw: true,
            children: render_tree,
            cache_self_size: Cell::new(None),
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct ColumnRenderNode<R: 'static> {
    label: Option<String>,
    redraw: bool,
    children: Vec<Box<dyn Widget<R>>>,
    cache_self_size: Cell<Option<PxSize>>,
}

impl<'a, R: 'static> WidgetTrait<R> for ColumnRenderNode<R> {
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn widget_event(
        &self,
        event: &crate::events::UiEvent,
        parent_size: PxSize,
        context: &ApplicationContext,
    ) -> UiEventResult<R> {
        // todo
        todo!()
    }

    fn update_render_tree(&mut self, dom: &dyn Dom<R>) -> Result<(), ()> {
        // todo
        if (*dom).type_id() != (*self).type_id() {
            Err(())
        } else {
            let dom = dom.as_any().downcast_ref::<Column<R>>().unwrap();
            self.children.clear();
            for child in dom.children.iter() {
                self.children.push(child.build_render_tree());
            }
            Ok(())
        }
    }

    fn compare(&self, dom: &dyn Dom<R>) -> super::DomComPareResult {
        if (*dom).type_id() != (*self).type_id() {
            return DomComPareResult::Different;
        }

        // todo
        DomComPareResult::Different
    }
}

impl<R> RenderingTrait for ColumnRenderNode<R> {
    fn size(&self) -> crate::types::size::Size {
        Size {
            width: SizeUnit::Content(1.0),
            height: SizeUnit::Content(1.0),
        }
    }

    fn px_size(
        &self,
        _: crate::types::size::PxSize,
        context: &ApplicationContext,
    ) -> crate::types::size::PxSize {
        let mut width: f32 = 0.0;
        let mut height_px: f32 = 0.0;
        let mut height_percent: f32 = 0.0;

        for child in &self.children {
            let child_std_size = StdSize::from_size(child.size(), context);

            match child_std_size.width {
                StdSizeUnit::Pixel(px) => width = width.max(px),
                StdSizeUnit::Percent(_) => (),
                StdSizeUnit::None => width = width.max(child.default_size().width),
            }

            match child_std_size.height {
                StdSizeUnit::Pixel(px) => height_px += px,
                StdSizeUnit::Percent(percent) => height_percent += percent,
                StdSizeUnit::None => height_px += child.default_size().height,
            }
        }

        let height = height_px / (1.0 - height_percent);

        self.cache_self_size.set(Some(PxSize { width, height }));
        crate::types::size::PxSize { width, height }
    }

    fn default_size(&self) -> crate::types::size::PxSize {
        todo!()
    }

    fn render(
        &mut self,
        s: &rayon::Scope,
        parent_size: PxSize,
        affine: na::Matrix4<f32>,
        encoder: &mut RendererCommandEncoder,
    ) {
        let current_size = self.px_size(parent_size, encoder.get_context());

        let mut accumulated_height: f32 = 0.0;
        for child in &mut self.children {
            let child_px_size = child.px_size(current_size, encoder.get_context());
            let child_affine =
                na::Matrix4::new_translation(&na::Vector3::new(0.0, -accumulated_height, 0.0))
                    * affine;
            child.render(s, child_px_size, child_affine, encoder);
            accumulated_height += child_px_size.height;
        }
    }
}
