use nalgebra as na;
use std::{any::Any, cell::Cell};

use super::{
    DomComPareResult, DomNode, RenderItem, RenderingTrait, /* SubNode,*/ Widget, WidgetTrait,
};
use crate::{
    application_context::ApplicationContext,
    events::WidgetEventResult,
    render::RenderCommandEncoder,
    types::size::{PxSize, Size, SizeUnit, StdSize, StdSizeUnit},
};

pub struct Column<R: 'static> {
    children: Vec<Box<dyn DomNode<R>>>,
}

impl<R: 'static> Column<R> {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    pub fn vec(vec: Vec<Box<dyn DomNode<R>>>) -> Self {
        Self { children: vec }
    }

    pub fn push(&mut self, child: Box<dyn DomNode<R>>) {
        self.children.push(child);
    }
}

impl<R: 'static> DomNode<R> for Column<R> {
    fn build_render_tree(&self) -> Box<dyn Widget<R>> {
        let mut render_tree = Vec::new();

        for child in &self.children {
            render_tree.push(child.build_render_tree());
        }

        Box::new(ColumnRenderNode {
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
    redraw: bool,
    children: Vec<Box<dyn Widget<R>>>,
    cache_self_size: Cell<Option<PxSize>>,
}

impl<'a, R: 'static> WidgetTrait<R> for ColumnRenderNode<R> {
    fn widget_event(&self, event: &crate::events::WidgetEvent) -> WidgetEventResult<R> {
        // todo
        todo!()
    }

    fn update_render_tree(&mut self, dom: &dyn DomNode<R>) -> Result<(), ()> {
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

    fn compare(&self, dom: &dyn DomNode<R>) -> super::DomComPareResult {
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
        parent_size: PxSize,
        affine: na::Matrix4<f32>,
        encoder: &mut RenderCommandEncoder,
    ) {
        let current_size = self.px_size(parent_size, encoder.get_context());

        let mut accumulated_height: f32 = 0.0;
        for child in &mut self.children {
            let child_px_size = child.px_size(current_size, encoder.get_context());
            let child_affine =
                na::Matrix4::new_translation(&na::Vector3::new(0.0, -accumulated_height, 0.0))
                    * affine;
            child.render(child_px_size, child_affine, encoder);
            accumulated_height += child_px_size.height;
        }
    }
}
