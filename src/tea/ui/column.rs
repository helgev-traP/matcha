use crate::{application_context::ApplicationContext, types::size::OptionPxSize};

use super::{DomNode, RenderNode};

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
    fn build_render_tree(&self) -> Box<dyn RenderNode<R>> {
        let mut render_tree = Vec::new();

        for child in &self.children {
            render_tree.push(child.build_render_tree());
        }

        Box::new(ColumnRenderNode {
            children: render_tree,
        })
    }
}

pub struct ColumnRenderNode<R: 'static> {
    children: Vec<Box<dyn RenderNode<R>>>,
}

impl<R: 'static> RenderNode<R> for ColumnRenderNode<R> {
    fn update_render_tree(&self, _new: &dyn DomNode<R>) {
        unimplemented!()
    }

    fn render(
        &mut self,
        app_context: &ApplicationContext,
        parent_size: OptionPxSize,
    ) -> super::RenderObject {
        todo!()
    }

    fn widget_event(&self, event: &crate::events::WidgetEvent) -> Option<R> {
        todo!()
    }

    fn compare(&self, dom: &dyn DomNode<R>) -> super::DomComPareResult {
        todo!()
    }
}
