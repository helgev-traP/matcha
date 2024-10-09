use std::{cell::RefCell, sync::{Arc, RwLock}};

use nalgebra as na;

use crate::{application_context::ApplicationContext, types::size::OptionPxSize};

use super::{DomNode, RenderNode, RenderTrait, SubNode};

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

    pub fn push(mut self, child: Box<dyn DomNode<R>>) -> Self {
        self.children.push(child);
        self
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
        }))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct ColumnRenderNode<R: 'static> {
    redraw: bool,
    children: Vec<RenderNode<R>>,
}

impl<'a, R: 'static> RenderTrait<R> for ColumnRenderNode<R> {
    fn redraw(&self) -> bool {
        self.redraw
    }

    fn sub_nodes(&self) -> Vec<SubNode<R>> {
        let mut sub_nodes = Vec::new();

        for child in &self.children {
            sub_nodes.push(SubNode {
                affine: na::Matrix4::identity(),
                node: child.clone(),
            });
        }

        sub_nodes
    }

    fn size(&self) -> crate::types::size::Size {
        // collect all children size that be able to know the actual size
        let children_with_percented_size: Vec<&RenderNode<R>> = Vec::new();

        // todo <<<<<<<<<<<<<<<<<<<<< ここから
        todo!()
    }

    fn px_size(
        &self,
        parent_size: crate::types::size::PxSize,
        context: &ApplicationContext,
    ) -> crate::types::size::PxSize {
        todo!()
    }

    fn default_size(&self) -> crate::types::size::PxSize {
        todo!()
    }

    fn render(
        &self,
        app_context: &ApplicationContext,
        parent_size: crate::types::size::PxSize,
    ) -> super::RenderItem {
        todo!()
    }

    fn widget_event(&self, event: &crate::events::WidgetEvent) -> Option<R> {
        todo!()
    }

    fn update_render_tree(&mut self, dom: &dyn DomNode<R>) {
        todo!()
    }

    fn compare(&self, dom: &dyn DomNode<R>) -> super::DomComPareResult {
        todo!()
    }
}
