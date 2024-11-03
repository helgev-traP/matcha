use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};

use super::{
    application_context::ApplicationContext,
    events::{UiEvent, UiEventResult},
    renderer::RendererCommandEncoder,
    types::size::PxSize,
    ui::{Dom, DomComPareResult, RenderingTrait, Widget, WidgetTrait},
};

pub struct Component<Model, Message, OuterResponse, InnerResponse>
where
    Model: Send + 'static,
    OuterResponse: 'static,
    InnerResponse: 'static,
{
    label: Option<String>,

    model: Arc<Mutex<Model>>,
    model_updated: Arc<Mutex<bool>>,
    fn_update: fn(ComponentAccess<Model>, Message),
    fn_local_update:
        fn(&ComponentAccess<Model>, UiEventResult<InnerResponse>) -> UiEventResult<OuterResponse>,
    fn_view: fn(&Model) -> Box<dyn Dom<InnerResponse>>,

    render_tree: Option<Arc<Mutex<Box<dyn Widget<InnerResponse>>>>>,
}

impl<Model, Message, OuterResponse, InnerResponse>
    Component<Model, Message, OuterResponse, InnerResponse>
where
    Model: Send + 'static,
    OuterResponse: 'static,
    InnerResponse: 'static,
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

impl<Model, Message, OuterResponse, InnerResponse>
    Component<Model, Message, OuterResponse, InnerResponse>
where
    Model: Send + 'static,
    OuterResponse: 'static,
    InnerResponse: 'static,
{
    pub fn label(&self) -> Option<&String> {
        self.label.as_ref()
    }

    pub async fn update(&mut self, message: Message) {
        (self.fn_update)(
            ComponentAccess {
                model: self.model.clone(),
                model_updated: self.model_updated.clone(),
            },
            message,
        );

        if *self.model_updated.lock().await {
            self.update_render_tree().await;
            *self.model_updated.lock().await = false;
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

    async fn update_render_tree(&mut self) {
        let dom = (self.fn_view)(&*self.model.lock().await);

        if let Some(ref mut render_tree) = self.render_tree {
            if let Ok(_) = render_tree.lock().await.update_render_tree(&*dom) {
                return;
            }
            self.render_tree = Some(Arc::new(Mutex::new(dom.build_render_tree())));
        } else {
            self.render_tree = Some(Arc::new(Mutex::new(dom.build_render_tree())));
        }
    }

    pub async fn view(&mut self) -> Option<Arc<dyn Dom<OuterResponse>>> {
        if let None = self.render_tree {
            self.update_render_tree().await;
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

pub struct ComponentAccess<Model>
where
    Model: Send + 'static,
{
    model: Arc<Mutex<Model>>,
    model_updated: Arc<Mutex<bool>>,
}

impl<Model> Clone for ComponentAccess<Model>
where
    Model: Send + 'static,
{
    fn clone(&self) -> Self {
        Self {
            model: self.model.clone(),
            model_updated: self.model_updated.clone(),
        }
    }
}

impl<Model> ComponentAccess<Model>
where
    Model: Send + 'static,
{
    pub async fn model_ref(&self) -> MutexGuard<Model> {
        self.model.lock().await
    }

    pub async fn model_mut(&mut self) -> MutexGuard<Model> {
        *self.model_updated.lock().await = true;
        self.model.lock().await
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
    render_tree: Arc<Mutex<Box<dyn Widget<InnerResponse>>>>,
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

pub struct ComponentRenderNode<Model, OuterResponse, InnerResponse>
where
    Model: Send + 'static,
    OuterResponse: 'static,
    InnerResponse: 'static,
{
    label: Option<String>,
    component_model: ComponentAccess<Model>,
    local_update_component:
        fn(&ComponentAccess<Model>, UiEventResult<InnerResponse>) -> UiEventResult<OuterResponse>,
    node: Arc<Mutex<Box<dyn Widget<InnerResponse>>>>,
}

#[async_trait::async_trait]
impl<Model, O, I> WidgetTrait<O> for ComponentRenderNode<Model, O, I>
where
    Model: Send + 'static,
    O: 'static,
    I: 'static,
{
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    async fn widget_event(
        &mut self,
        event: &UiEvent,
        parent_size: PxSize,
        context: &ApplicationContext,
    ) -> UiEventResult<O> {
        (self.local_update_component)(
            &self.component_model,
            self.node
                .lock()
                .await
                .widget_event(event, parent_size, context)
                .await,
        )
    }

    async fn is_inside(
        &self,
        position: [f32; 2],
        parent_size: PxSize,
        context: &ApplicationContext,
    ) -> bool {
        self.node
            .lock()
            .await
            .is_inside(position, parent_size, context)
            .await
    }

    fn compare(&self, _: &dyn Dom<O>) -> DomComPareResult {
        DomComPareResult::Different
    }

    fn update_render_tree(&mut self, _: &dyn Dom<O>) -> Result<(), ()> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl<Model: Send, OuterResponse, InnerResponse> RenderingTrait
    for ComponentRenderNode<Model, OuterResponse, InnerResponse>
{
    async fn size(&self) -> super::types::size::Size {
        self.node.lock().await.size().await
    }

    async fn px_size(&self, parent_size: PxSize, context: &ApplicationContext) -> PxSize {
        self.node.lock().await.px_size(parent_size, context).await
    }

    async fn default_size(&self) -> super::types::size::PxSize {
        self.node.lock().await.default_size().await
    }

    async fn render(
        &mut self,
        parent_size: PxSize,
        affine: nalgebra::Matrix4<f32>,
        encoder: RendererCommandEncoder,
    ) {
        self.node.lock().await.render(parent_size, affine, encoder).await;
    }
}
