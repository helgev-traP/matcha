use crate::{application_context::ApplicationContext, types::size::OptionPxSize};

use super::{DomNode, RenderNode, RenderTrait};

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

        Box::new(ColumnRenderNode {
            children: render_tree,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct ColumnRenderNode<R: 'static> {
    children: Vec<RenderNode<R>>,
}

impl<R: 'static> RenderTrait<R> for ColumnRenderNode<R> {
    fn redraw(&self) -> bool {
        true
    }

    fn sub_nodes(&self) -> Vec<super::SubNode<R>> {
        todo!()
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
        &mut self,
        app_context: &ApplicationContext,
        parent_size: crate::types::size::PxSize,
    ) -> super::RenderItem {
        todo!()
    }

    fn widget_event(&self, event: &crate::events::WidgetEvent) -> Option<R> {
        todo!()
    }

    fn update_render_tree(&self, dom: &dyn DomNode<R>) {
        todo!()
    }

    fn compare(&self, dom: &dyn DomNode<R>) -> super::DomComPareResult {
        todo!()
    }
}
