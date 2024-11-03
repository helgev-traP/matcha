use std::{cell::RefCell, rc::Rc, sync::{Arc, Mutex, MutexGuard}};

use super::{
    application_context::ApplicationContext,
    events::UiEventResult,
    renderer::RendererCommandEncoder,
    types::size::PxSize,
    ui::{Dom, DomComPareResult, RenderingTrait, Widget, WidgetTrait},
};

pub struct Component<Model, Message, OuterResponse, InnerResponse> {
    label: Option<String>,

    model: Arc<Mutex<Model>>,
    model_updated: Arc<Mutex<bool>>,
    fn_update: fn(ComponentAccess<Model>, Message),
    fn_local_update:
        fn(&ComponentAccess<Model>, UiEventResult<InnerResponse>) -> UiEventResult<OuterResponse>,
    fn_view: fn(&Model) -> Box<dyn Dom<InnerResponse>>,

    render_tree: Option<Rc<RefCell<Box<dyn Widget<InnerResponse>>>>>,
}

impl<Model: Send + 'static, Message, OuterResponse: 'static, InnerResponse: 'static>
    Component<Model, Message, OuterResponse, InnerResponse>
{
    pub fn new(
        label: Option<String>,
        model: Model,
        update: fn(ComponentAccess<Model>, Message),
        view: fn(&Model) -> Box<dyn Dom<InnerResponse>>,
    ) -> Self {
        Self {
            label,
            model: Arc::new(Mutex::new(model)),
            model_updated: Arc::new(Mutex::new(true)),
            fn_update: update,
            fn_local_update: |_, _| Default::default(),
            fn_view: view,
            render_tree: None,
        }
    }

    pub fn component_update(
        mut self,
        component_update: fn(
            &ComponentAccess<Model>,
            UiEventResult<InnerResponse>,
        ) -> UiEventResult<OuterResponse>,
    ) -> Self {
        self.fn_local_update = component_update;
        self
    }
}

impl<Model: Send + 'static, Message, OuterResponse: 'static, InnerResponse: 'static>
    Component<Model, Message, OuterResponse, InnerResponse>
{
    pub fn label(&self) -> Option<&String> {
        self.label.as_ref()
    }

    pub fn update(&mut self, message: Message) {
        (self.fn_update)(
            ComponentAccess {
                model: self.model.clone(),
                model_updated: self.model_updated.clone(),
            },
            message,
        );

        if *self.model_updated.lock().unwrap() {
            self.update_render_tree();
            *self.model_updated.lock().unwrap() = false;
        }
    }

    fn update_local(
        &mut self,
        event: UiEventResult<InnerResponse>,
    ) -> UiEventResult<OuterResponse> {
        (self.fn_local_update)(
            &ComponentAccess {
                model: self.model.clone(),
                model_updated: self.model_updated.clone(),
            },
            event,
        )
    }

    fn update_render_tree(&mut self) {
        let dom = (self.fn_view)(&*self.model.lock().unwrap());

        if let Some(ref mut render_tree) = self.render_tree {
            if let Ok(_) = render_tree.borrow_mut().update_render_tree(&*dom) {
                return;
            }
            self.render_tree = Some(Rc::new(RefCell::new(dom.build_render_tree())));
        } else {
            self.render_tree = Some(Rc::new(RefCell::new(dom.build_render_tree())));
        }
    }

    pub fn view(&mut self) -> Option<Arc<dyn Dom<OuterResponse>>> {
        if let None = self.render_tree {
            self.update_render_tree();
        }
        Some(Arc::new(ComponentDom {
            label: self.label.clone(),
            component_model: ComponentAccess {
                model: self.model.clone(),
                model_updated: self.model_updated.clone(),
            },
            local_update_component: self.fn_local_update,
            render_tree: self.render_tree.as_ref().unwrap().clone(),
        }))
    }
}

pub struct ComponentAccess<Model> {
    model: Arc<Mutex<Model>>,
    model_updated: Arc<Mutex<bool>>,
}

impl<Model> Clone for ComponentAccess<Model> {
    fn clone(&self) -> Self {
        Self {
            model: self.model.clone(),
            model_updated: self.model_updated.clone(),
        }
    }
}

impl<Model> ComponentAccess<Model> {
    pub fn model_ref(&self) -> MutexGuard<Model> {
        self.model.lock().unwrap()
    }

    pub fn model_mut(&mut self) -> MutexGuard<Model> {
        *self.model_updated.lock().unwrap() = true;
        self.model.lock().unwrap()
    }
}

pub struct ComponentDom<Model, OuterResponse, InnerResponse>
where
    Model: Send + 'static,
    OuterResponse: 'static,
    InnerResponse: 'static,
{
    label: Option<String>,
    component_model: ComponentAccess<Model>,
    local_update_component:
        fn(&ComponentAccess<Model>, UiEventResult<InnerResponse>) -> UiEventResult<OuterResponse>,
    render_tree: Rc<RefCell<Box<dyn Widget<InnerResponse>>>>,
}

impl<Model: Send, OuterResponse, InnerResponse> Dom<OuterResponse>
    for ComponentDom<Model, OuterResponse, InnerResponse>
where
    Model: 'static,
    OuterResponse: 'static,
    InnerResponse: 'static,
{
    fn build_render_tree(&self) -> Box<dyn Widget<OuterResponse>> {
        Box::new(ComponentRenderNode {
            label: self.label.clone(),
            component_model: self.component_model.clone(),
            local_update_component: self.local_update_component,
            node: self.render_tree.clone(),
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct ComponentRenderNode<Model, OuterResponse: 'static, InnerResponse: 'static> {
    label: Option<String>,
    component_model: ComponentAccess<Model>,
    local_update_component:
        fn(&ComponentAccess<Model>, UiEventResult<InnerResponse>) -> UiEventResult<OuterResponse>,
    node: Rc<RefCell<Box<dyn Widget<InnerResponse>>>>,
}

impl<Model, O, I> WidgetTrait<O> for ComponentRenderNode<Model, O, I> {
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn widget_event(
        &mut self,
        event: &super::events::UiEvent,
        parent_size: PxSize,
        context: &ApplicationContext,
    ) -> UiEventResult<O> {
        (self.local_update_component)(
            &self.component_model,
            self.node
                .borrow_mut()
                .widget_event(event, parent_size, context),
        )
    }

    fn is_inside(
        &self,
        position: [f32; 2],
        parent_size: PxSize,
        context: &ApplicationContext,
    ) -> bool {
        self.node
            .borrow()
            .is_inside(position, parent_size, context)
    }

    fn compare(&self, _: &dyn Dom<O>) -> DomComPareResult {
        DomComPareResult::Different
    }

    fn update_render_tree(&mut self, _: &dyn Dom<O>) -> Result<(), ()> {
        Ok(())
    }
}

impl<Model: Send, OuterResponse, InnerResponse> RenderingTrait
    for ComponentRenderNode<Model, OuterResponse, InnerResponse>
{
    fn size(&self) -> super::types::size::Size {
        self.node.borrow().size()
    }

    fn px_size(&self, parent_size: PxSize, context: &ApplicationContext) -> PxSize {
        self.node.borrow().px_size(parent_size, context)
    }

    fn default_size(&self) -> super::types::size::PxSize {
        self.node.borrow().default_size()
    }

    fn render(
        &mut self,
        parent_size: PxSize,
        affine: nalgebra::Matrix4<f32>,
        encoder: RendererCommandEncoder,
    )
    {
        self.node.borrow_mut().render(parent_size, affine, encoder);
    }
}
