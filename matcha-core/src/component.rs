use std::{
    any::Any,
    sync::{Arc, atomic::AtomicBool},
};

use tokio::sync::{Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{
    observer::{Observer, ObserverSender},
    ui::WidgetContext,
};

use super::{
    types::range::CoverRange,
    ui::{Background, Dom, DomComPareResult, Object, UpdateWidgetError, Widget},
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
    updated: AtomicBool,
    observer_sender: Mutex<Option<ObserverSender>>,
}

impl UpdateFlag {
    fn new(b: bool) -> Self {
        Self {
            updated: AtomicBool::new(b),
            observer_sender: Mutex::new(None),
        }
    }

    /// When the model is updated, this function should be called to notify the observer.
    async fn set_to_true(&self) {
        self.updated
            .store(true, std::sync::atomic::Ordering::Release);
        if let Some(sender) = &mut *self.observer_sender.lock().await {
            sender.send_update();
        }
    }

    fn set_to_false(&self) {
        self.updated
            .store(false, std::sync::atomic::Ordering::Release);
    }

    /// Create an observer receiver. Also reset the update flag to false.
    async fn set_observer(&self, observer: &Observer) {
        let mut observer_sender = self.observer_sender.lock().await;
        *observer_sender = Some(observer.sender());
    }

    async fn is_updated(&self) -> bool {
        self.updated.load(std::sync::atomic::Ordering::Acquire)
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
            model_update_flag: Arc::new(UpdateFlag::new(false)),
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
    // make update_flag to `false` when `build_widget_tree` or `update_widget_tree` called.
    fn build_widget_tree(&self) -> Box<dyn Widget<Response>> {
        self.model_accessor.update_flag.set_to_false();

        Box::new(ComponentWidget {
            label: self.label.clone(),
            model_accessor: self.model_accessor.clone(),
            react_fn: self.react_fn,
            widget: self.dom.build_widget_tree(),
        })
    }

    async fn set_observer(&self, observer: &Observer) {
        self.update_flag.set_observer(observer).await;
        self.dom.set_observer(observer).await;
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

    // make update_flag to false when `build_widget_tree` or `update_widget_tree` called.
    async fn update_widget_tree(
        &mut self,
        _: bool,
        dom: &dyn Dom<Response>,
    ) -> Result<(), UpdateWidgetError> {
        // todo: optimize
        if let Some(component_dom) =
            (dom as &dyn Any).downcast_ref::<ComponentDom<Model, Response, InnerResponse>>()
        {
            let is_component_updated = component_dom.update_flag.is_updated().await;

            self.model_accessor.update_flag.set_to_false();

            // todo: optimize and update label

            self.widget
                .update_widget_tree(is_component_updated, component_dom.dom.as_ref())
                .await
        } else {
            return Err(UpdateWidgetError::TypeMismatch);
        }
    }

    fn compare(&self, dom: &dyn Dom<Response>) -> DomComPareResult {
        // todo: optimize
        if let Some(component_dom) =
            (dom as &dyn Any).downcast_ref::<ComponentDom<Model, Response, InnerResponse>>()
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
        event: &super::events::Event,
        parent_size: [Option<f32>; 2],
        context: &WidgetContext,
    ) -> Option<Response> {
        self.widget
            .widget_event(event, parent_size, context)
            .and_then(|inner_response| (self.react_fn)(inner_response, self.model_accessor.clone()))
    }

    fn px_size(&mut self, parent_size: [Option<f32>; 2], context: &WidgetContext) -> [f32; 2] {
        self.widget.px_size(parent_size, context)
    }

    fn is_inside(
        &mut self,
        position: [f32; 2],
        parent_size: [Option<f32>; 2],
        context: &WidgetContext,
    ) -> bool {
        self.widget.is_inside(position, parent_size, context)
    }

    fn cover_range(
        &mut self,
        parent_size: [Option<f32>; 2],
        context: &WidgetContext,
    ) -> CoverRange<f32> {
        self.widget.cover_range(parent_size, context)
    }

    fn need_rerendering(&self) -> bool {
        self.widget.need_rerendering()
    }

    fn render(
        &mut self,
        render_pass: &mut wgpu::RenderPass<'_>,
        target_size: [u32; 2],
        target_format: wgpu::TextureFormat,
        parent_size: [Option<f32>; 2],
        background: Background,
        context: &WidgetContext,
    ) -> Object {
        self.widget.render(
            render_pass,
            target_size,
            target_format,
            parent_size,
            background,
            context,
        )
    }
}
