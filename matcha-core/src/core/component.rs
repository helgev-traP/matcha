use std::sync::Arc;

use tokio::sync::{Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};

use super::{
    context::SharedContext,
    observer::{ObserverReceiver, ObserverSender, create_observer_ch},
    types::range::Range2D,
    ui::{Dom, DomComPareResult, Object, UpdateWidgetError, Widget},
};

// MARK: - ModelAccessor

pub struct ModelAccessor<Model: 'static> {
    model: Arc<RwLock<Model>>,
    update_flag: Arc<UpdateFlag>,
}

impl<Model: 'static> Clone for ModelAccessor<Model> {
    fn clone(&self) -> Self {
        Self {
            model: Arc::clone(&self.model),
            update_flag: Arc::clone(&self.update_flag),
        }
    }
}

impl<Model: 'static> ModelAccessor<Model> {
    pub async fn get_ref(&self) -> RwLockReadGuard<Model> {
        self.model.read().await
    }

    pub async fn update<F>(&self, f: F)
    where
        F: FnOnce(RwLockWriteGuard<Model>),
    {
        // ensure update function finish before change the update flag
        let model = self.model.write().await;
        f(model);
        self.update_flag.set_to_true().await;
    }
}

// MARK: - UpdateFlag

struct UpdateFlag {
    updated: Mutex<bool>, // todo: consider use `AtomicBool` or std-Mutex
    observer_sender: Mutex<Option<ObserverSender>>,
}

impl UpdateFlag {
    fn new() -> Self {
        Self {
            updated: Mutex::new(true),
            observer_sender: Mutex::new(None),
        }
    }

    /// When the model is updated, this function should be called to notify the observer.
    async fn set_to_true(&self) {
        let mut updated = self.updated.lock().await;
        *updated = true;
        if let Some(sender) = &mut *self.observer_sender.lock().await {
            sender.send_update();
        }
    }

    /// Create an observer receiver. Also reset the update flag to false.
    async fn make_observer(&self) -> ObserverReceiver {
        // update the flag to false
        {
            let mut updated = self.updated.lock().await;
            *updated = false;
        }

        // make observer
        let (sender, receiver) = create_observer_ch();
        let mut observer_sender = self.observer_sender.lock().await;
        *observer_sender = Some(sender);
        receiver
    }

    async fn is_updated(&self) -> bool {
        let updated = self.updated.lock().await;
        *updated
    }
}

// MARK: - Component

pub struct Component<
    Model: Send + Sync + 'static,
    Message,
    Response: 'static,
    InnerResponse: 'static = Response,
> {
    label: Option<String>,

    // model
    model: Arc<RwLock<Model>>,
    model_update_flag: Arc<UpdateFlag>,

    // setup function
    setup_fn: fn(ModelAccessor<Model>),
    // model update function
    update_fn: fn(Message, ModelAccessor<Model>),
    // react function
    react_fn: fn(InnerResponse, ModelAccessor<Model>) -> Option<Response>,
    // elm view function
    view_fn: fn(&Model) -> Box<dyn Dom<InnerResponse>>,
}

/// constructor
impl<Model: Send + Sync + 'static, Message, Response: 'static, InnerResponse: 'static>
    Component<Model, Message, Response, InnerResponse>
{
    pub fn new(
        label: Option<&str>,
        model: Model,
        view: fn(&Model) -> Box<dyn Dom<InnerResponse>>,
    ) -> Self {
        Self {
            label: label.map(|s| s.to_string()),
            model: Arc::new(RwLock::new(model)),
            model_update_flag: Arc::new(UpdateFlag::new()),
            setup_fn: |_: ModelAccessor<Model>| {},
            update_fn: |_: Message, _: ModelAccessor<Model>| {},
            react_fn: |_: InnerResponse, _: ModelAccessor<Model>| None,
            view_fn: view,
        }
    }

    pub fn setup_fn(mut self, f: fn(ModelAccessor<Model>)) -> Self {
        self.setup_fn = f;
        self
    }

    pub fn update_fn(mut self, f: fn(Message, ModelAccessor<Model>)) -> Self {
        self.update_fn = f;
        self
    }

    pub fn react_fn<NewResponse: 'static>(
        self,
        f: fn(InnerResponse, ModelAccessor<Model>) -> Option<NewResponse>,
    ) -> Component<Model, Message, NewResponse, InnerResponse> {
        Component {
            label: self.label,
            model: self.model,
            model_update_flag: self.model_update_flag,
            setup_fn: self.setup_fn,
            update_fn: self.update_fn,
            react_fn: f,
            view_fn: self.view_fn,
        }
    }
}

/// functional methods
impl<Model: Send + Sync + 'static, Message, Response: 'static, InnerResponse: 'static>
    Component<Model, Message, Response, InnerResponse>
{
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    pub fn update(&self, message: Message) {
        let model_accessor = ModelAccessor {
            model: Arc::clone(&self.model),
            update_flag: Arc::clone(&self.model_update_flag),
        };

        (self.update_fn)(message, model_accessor);
    }

    pub async fn view(&self) -> Box<dyn Dom<Response>> {
        Box::new(ComponentDom {
            label: self.label.clone(),
            update_flag: Arc::clone(&self.model_update_flag),
            model_accessor: ModelAccessor {
                model: Arc::clone(&self.model),
                update_flag: Arc::clone(&self.model_update_flag),
            },
            react_fn: self.react_fn,
            dom: (self.view_fn)(&*self.model.read().await),
        })
    }
}

// MARK: - ComponentDom

pub struct ComponentDom<Model: Sync + 'static, Response: 'static, InnerResponse: 'static> {
    label: Option<String>,

    update_flag: Arc<UpdateFlag>,
    model_accessor: ModelAccessor<Model>,
    react_fn: fn(InnerResponse, ModelAccessor<Model>) -> Option<Response>,

    dom: Box<dyn Dom<InnerResponse>>,
}

#[async_trait::async_trait]
impl<Model: Sync + Send + 'static, Response: 'static, InnerResponse: 'static> Dom<Response>
    for ComponentDom<Model, Response, InnerResponse>
{
    fn build_widget_tree(&self) -> Box<dyn Widget<Response>> {
        Box::new(ComponentWidget {
            label: self.label.clone(),
            model_accessor: self.model_accessor.clone(),
            react_fn: self.react_fn,
            widget: self.dom.build_widget_tree(),
        })
    }

    async fn collect_observer(&self) -> super::observer::Observer {
        let mut observer = self.dom.collect_observer().await;
        observer.add_receiver(self.update_flag.make_observer().await);
        observer
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// MARK: - ComponentWidget

pub struct ComponentWidget<Model: Sync + 'static, Response: 'static, InnerResponse: 'static> {
    label: Option<String>,

    model_accessor: ModelAccessor<Model>,
    react_fn: fn(InnerResponse, ModelAccessor<Model>) -> Option<Response>,

    widget: Box<dyn Widget<InnerResponse>>,
}

#[async_trait::async_trait]
impl<Model: Sync + Send + 'static, Response: 'static, InnerResponse: 'static> Widget<Response>
    for ComponentWidget<Model, Response, InnerResponse>
{
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    async fn update_widget_tree(
        &mut self,
        _: bool,
        dom: &dyn Dom<Response>,
    ) -> Result<(), UpdateWidgetError> {
        // todo: optimize
        if let Some(component_dom) = dom
            .as_any()
            .downcast_ref::<ComponentDom<Model, Response, InnerResponse>>()
        {
            let is_component_updated = component_dom.update_flag.is_updated().await;

            self.widget
                .update_widget_tree(is_component_updated, component_dom.dom.as_ref())
                .await
        } else {
            return Err(UpdateWidgetError::TypeMismatch);
        }
    }

    fn compare(&self, dom: &dyn Dom<Response>) -> DomComPareResult {
        // todo: optimize
        if let Some(component_dom) = dom
            .as_any()
            .downcast_ref::<ComponentDom<Model, Response, InnerResponse>>()
        {
            if self.label == component_dom.label {
                DomComPareResult::Same
            } else {
                DomComPareResult::Different
            }
        } else {
            DomComPareResult::Different
        }
    }

    fn widget_event(
        &mut self,
        event: &super::events::UiEvent,
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
    ) -> Option<Response> {
        self.widget
            .widget_event(event, parent_size, context)
            .and_then(|inner_response| (self.react_fn)(inner_response, self.model_accessor.clone()))
    }

    fn px_size(&mut self, parent_size: [Option<f32>; 2], context: &SharedContext) -> [f32; 2] {
        self.widget.px_size(parent_size, context)
    }

    fn is_inside(
        &mut self,
        position: [f32; 2],
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
    ) -> bool {
        self.widget.is_inside(position, parent_size, context)
    }

    fn draw_range(
        &mut self,
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
    ) -> Option<Range2D<f32>> {
        self.widget.draw_range(parent_size, context)
    }

    fn cover_area(
        &mut self,
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
    ) -> Option<Range2D<f32>> {
        self.widget.cover_area(parent_size, context)
    }

    fn redraw(&self) -> bool {
        self.widget.redraw()
    }

    fn render(
        &mut self,
        parent_size: [Option<f32>; 2],
        background_view: &wgpu::TextureView,
        background_range: Range2D<f32>,
        context: &SharedContext,
        renderer: &super::renderer::Renderer,
    ) -> Vec<Object> {
        self.widget.render(
            parent_size,
            background_view,
            background_range,
            context,
            renderer,
        )
    }
}
