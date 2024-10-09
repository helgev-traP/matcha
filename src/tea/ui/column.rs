use std::{
    any::Any, cell::Cell, sync::{Arc, RwLock}
};

use nalgebra as na;

use crate::{
    application_context::ApplicationContext,
    types::size::{PxSize, Size, SizeUnit, StdSize, StdSizeUnit},
};

use super::{DomComPareResult, DomNode, RenderItem, RenderNode, RenderTrait, SubNode};

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
    fn build_render_tree(&self) -> RenderNode<R> {
        let mut render_tree = Vec::new();

        for child in &self.children {
            render_tree.push(child.build_render_tree());
        }

        Arc::new(RwLock::new(ColumnRenderNode {
            redraw: true,
            children: render_tree,
            cache_self_size: Cell::new(None),
        }))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct ColumnRenderNode<R: 'static> {
    redraw: bool,
    children: Vec<RenderNode<R>>,
    cache_self_size: Cell<Option<PxSize>>,
}

impl<'a, R: 'static> RenderTrait<R> for ColumnRenderNode<R> {
    fn redraw(&self) -> bool {
        self.redraw
    }

    fn sub_nodes(&self, parent_size: PxSize, context: &ApplicationContext) -> Vec<SubNode<R>> {
        let mut sub_nodes = Vec::new();

        if self.cache_self_size.get().is_none() {
            self.cache_self_size.set(Some(self.px_size(parent_size, context)));
        }

        let mut accumulate_height: f32 = 0.0;

        for child in &self.children {
            sub_nodes.push(SubNode {
                affine: na::Matrix4::new_translation(&na::Vector3::new(
                    0.0,
                    -accumulate_height,
                    0.0,
                )) * na::Matrix4::identity(),
                node: child.clone(),
            });

            accumulate_height += child
                .read()
                .unwrap()
                .px_size(self.cache_self_size.get().unwrap(), context)
                .height;
        }

        sub_nodes
    }

    fn size(&self) -> crate::types::size::Size {
        Size {
            width: SizeUnit::Content(1.0),
            height: SizeUnit::Content(1.0),
        }
    }

    fn px_size(
        &self,
        parent_size: crate::types::size::PxSize,
        context: &ApplicationContext,
    ) -> crate::types::size::PxSize {
        let mut width: f32 = 0.0;
        let mut height_px: f32 = 0.0;
        let mut height_percent: f32 = 0.0;

        for child in &self.children {
            let child_std_size = StdSize::from_size(child.read().unwrap().size(), context);

            match child_std_size.width {
                StdSizeUnit::Pixel(px) => width = width.max(px),
                StdSizeUnit::Percent(percent) => (),
                StdSizeUnit::None => width = width.max(child.read().unwrap().default_size().width),
            }

            match child_std_size.height {
                StdSizeUnit::Pixel(px) => height_px += px,
                StdSizeUnit::Percent(percent) => height_percent += percent,
                StdSizeUnit::None => height_px += child.read().unwrap().default_size().height,
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
        &self,
        app_context: &ApplicationContext,
        parent_size: crate::types::size::PxSize,
    ) -> RenderItem {
        RenderItem {
            object: vec![],
            px_size: self.px_size(parent_size, app_context),
        }
    }

    fn widget_event(&self, event: &crate::events::WidgetEvent) -> Option<R> {
        // todo
        None
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
