use std::sync::Arc;

use super::{
    application_context::ApplicationContext, types::size::OptionPxSize, ui::{DomComPareResult, DomNode, RenderNode, RenderObject}
};

pub struct Component<Model, Message, R: 'static> {
    label: Option<String>,

    model: Model,
    model_updated: bool,
    fn_update: fn(ComponentAccess<Model>, Message),
    fn_view: fn(&Model) -> Box<dyn DomNode<R>>,

    render_tree: Option<Arc<std::sync::Mutex<Box<dyn RenderNode<R>>>>>,
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

        if let Some(render_tree) = &self.render_tree {
            render_tree.lock().unwrap().update_render_tree(&*dom);
        } else {
            self.render_tree = Some(Arc::new(std::sync::Mutex::new(dom.build_render_tree())));
        }
    }

    pub fn view(&mut self) -> Option<Box<dyn DomNode<R>>> {
        if let None = self.render_tree {
            self.update_render_tree();
        }
        Some(Box::new(ComponentDom {
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
    render_tree: Arc<std::sync::Mutex<Box<dyn RenderNode<R>>>>,
}

impl<R: 'static> DomNode<R> for ComponentDom<R> {
    fn always_refresh(&self) -> bool {
        true
    }

    fn build_render_tree(&self) -> Box<dyn RenderNode<R>> {
        Box::new(ComponentRenderNode {
            node: self.render_tree.clone(),
        })
    }
}

pub struct ComponentRenderNode<R: 'static> {
    node: Arc<std::sync::Mutex<Box<dyn RenderNode<R>>>>,
}

impl<R: 'static> RenderNode<R> for ComponentRenderNode<R> {
    fn render(&mut self, app_context: &ApplicationContext, parent_size: OptionPxSize) -> RenderObject<R> {
        self.node.lock().unwrap().render(app_context, parent_size)
    }

    fn widget_event(&self, event: &super::events::WidgetEvent) -> Option<R> {
        self.node.lock().unwrap().widget_event(event)
    }

    fn compare(&self, _: &dyn DomNode<R>) -> DomComPareResult {
        DomComPareResult::Different
    }

    fn update_render_tree(&self, _: &dyn DomNode<R>) {}
}
