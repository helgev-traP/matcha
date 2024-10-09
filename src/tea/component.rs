use std::{cell::RefCell, rc::Rc, sync::Arc};

use super::{
    application_context::ApplicationContext,
    types::size::PxSize,
    ui::{DomComPareResult, DomNode, RenderItem, RenderNode, RenderTrait, SubNode},
};

pub struct Component<Model, Message, R: 'static> {
    label: Option<String>,

    model: Model,
    model_updated: bool,
    fn_update: fn(ComponentAccess<Model>, Message),
    fn_view: fn(&Model) -> Box<dyn DomNode<R>>,

    render_tree: Option<Rc<RefCell<RenderNode<R>>>>,
}

impl<Model, Message, R: 'static> Component<Model, Message, R> {
    pub fn new(
        label: Option<String>,
        model: Model,
        update: fn(ComponentAccess<Model>, Message),
        view: fn(&Model) -> Box<dyn DomNode<R>>,
    ) -> Self {
        Self {
            label,
            model,
            model_updated: true,
            fn_update: update,
            fn_view: view,
            render_tree: None,
        }
    }

    pub fn label(&self) -> Option<&String> {
        self.label.as_ref()
    }

    pub fn update(&mut self, message: Message) {
        (self.fn_update)(
            ComponentAccess {
                model: &mut self.model,
                model_updated: &mut self.model_updated,
            },
            message,
        );

        if self.model_updated {
            self.update_render_tree();
            self.model_updated = false;
        }
    }

    fn update_render_tree(&mut self) {
        let dom = (self.fn_view)(&self.model);

        if let Some(ref render_tree) = self.render_tree {
            (**render_tree).borrow().update_render_tree(&*dom);
        } else {
            self.render_tree = Some(Rc::new(RefCell::new(dom.build_render_tree())));
        }
    }

    pub fn view(&mut self) -> Option<Arc<dyn DomNode<R>>> {
        if let None = self.render_tree {
            self.update_render_tree();
        }
        Some(Arc::new(ComponentDom {
            render_tree: self.render_tree.as_ref().unwrap().clone(),
        }))
    }
}

pub struct ComponentAccess<'a, Model> {
    model: &'a mut Model,
    model_updated: &'a mut bool,
}

impl<Model> ComponentAccess<'_, Model> {
    pub fn model_ref(&self) -> &Model {
        self.model
    }

    pub fn model_mut(&mut self) -> &mut Model {
        *self.model_updated = true;
        self.model
    }
}

pub struct ComponentDom<R: 'static> {
    render_tree: Rc<RefCell<RenderNode<R>>>,
}

impl<R: 'static> DomNode<R> for ComponentDom<R> {
    fn build_render_tree(&self) -> RenderNode<R> {
        Box::new(ComponentRenderNode {
            node: self.render_tree.clone(),
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct ComponentRenderNode<R: 'static> {
    node: Rc<RefCell<RenderNode<R>>>,
}

impl<R: 'static> RenderTrait<R> for ComponentRenderNode<R> {
    fn redraw(&self) -> bool {
        (*self.node).borrow().redraw()
    }

    fn render(
        &mut self,
        app_context: &ApplicationContext,
        parent_size: PxSize,
    ) -> RenderItem {
        self.node.borrow_mut().render(app_context, parent_size)
    }

    fn widget_event(&self, event: &super::events::WidgetEvent) -> Option<R> {
        (*self.node).borrow().widget_event(event)
    }

    fn compare(&self, _: &dyn DomNode<R>) -> DomComPareResult {
        DomComPareResult::Different
    }

    fn update_render_tree(&self, _: &dyn DomNode<R>) {}

    fn sub_nodes(&self) -> Vec<SubNode<R>> {
        (*self.node).borrow().sub_nodes()
    }

    fn size(&self) -> super::types::size::Size {
        (*self.node).borrow().size()
    }

    fn px_size(&self, parent_size: PxSize, context: &ApplicationContext) -> PxSize {
        (*self.node).borrow().px_size(parent_size, context)
    }

    fn default_size(&self) -> super::types::size::PxSize {
        (*self.node).borrow().default_size()
    }
}
