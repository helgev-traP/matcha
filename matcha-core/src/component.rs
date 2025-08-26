use std::{
    any::Any,
    sync::{Arc, atomic::AtomicBool},
};

use tokio::sync::{Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{
    device_event::DeviceEvent, render_node::RenderNode, ui::WidgetContext,
    update_flag::UpdateNotifier,
};

use super::{
    types::range::CoverRange,
    ui::{Background, Constraints, Dom, DomCompareResult, UpdateWidgetError, Widget},
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
    observer_sender: Mutex<Option<UpdateNotifier>>,
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
            sender.notify();
        }
    }

    fn set_to_false(&self) {
        self.updated
            .store(false, std::sync::atomic::Ordering::Release);
    }

    /// Create an observer receiver. Also reset the update flag to false.
    async fn set_update_notifier(&self, notifier: &UpdateNotifier) {
        let mut observer_sender = self.observer_sender.lock().await;
        *observer_sender = Some(notifier.clone());
    }

    fn is_updated(&self) -> bool {
        self.updated.load(std::sync::atomic::Ordering::Acquire)
    }
}

// MARK: - Component

pub struct Component<
    Model: Send + Sync + 'static,
    Message,
    Event: 'static,
    InnerEvent: 'static = Event,
> {
    label: Option<String>,

    // model
    model: Arc<RwLock<Model>>,
    model_update_flag: Arc<UpdateFlag>,

    // setup function
    setup: fn(ModelAccessor<Model>),
    // update model with message
    update: fn(&Message, ModelAccessor<Model>),
    // update model with device event
    device: fn(&DeviceEvent, ModelAccessor<Model>),
    // update model with inner event and can emit new event
    event: fn(InnerEvent, ModelAccessor<Model>) -> Option<Event>,
    // view function
    view: fn(&Model) -> Box<dyn Dom<InnerEvent>>,
}

/// constructor
impl<Model: Send + Sync + 'static, Message, Event: 'static, InnerEvent: 'static>
    Component<Model, Message, Event, InnerEvent>
{
    pub fn new(
        label: Option<&str>,
        model: Model,
        view: fn(&Model) -> Box<dyn Dom<InnerEvent>>,
    ) -> Self {
        Self {
            label: label.map(|s| s.to_string()),
            model: Arc::new(RwLock::new(model)),
            model_update_flag: Arc::new(UpdateFlag::new(false)),
            setup: |_: ModelAccessor<Model>| {},
            update: |_: &Message, _: ModelAccessor<Model>| {},
            device: |_: &DeviceEvent, _: ModelAccessor<Model>| {},
            event: |_: InnerEvent, _: ModelAccessor<Model>| None,
            view,
        }
    }

    pub fn setup_fn(mut self, f: fn(ModelAccessor<Model>)) -> Self {
        self.setup = f;
        self
    }

    pub fn update_fn(mut self, f: fn(&Message, ModelAccessor<Model>)) -> Self {
        self.update = f;
        self
    }

    pub fn event_fn(mut self, f: fn(&DeviceEvent, ModelAccessor<Model>)) -> Self {
        self.device = f;
        self
    }

    pub fn react_fn<NewEventType: 'static>(
        self,
        f: fn(InnerEvent, ModelAccessor<Model>) -> Option<NewEventType>,
    ) -> Component<Model, Message, NewEventType, InnerEvent> {
        Component {
            label: self.label,
            model: self.model,
            model_update_flag: self.model_update_flag,
            setup: self.setup,
            update: self.update,
            device: self.device,
            event: f,
            view: self.view,
        }
    }
}

/// functional methods
impl<Model: Send + Sync + 'static, Message, Event: 'static, InnerEvent: 'static>
    Component<Model, Message, Event, InnerEvent>
{
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    pub fn update(&self, message: &Message) {
        let model_accessor = ModelAccessor {
            model: Arc::clone(&self.model),
            update_flag: Arc::clone(&self.model_update_flag),
        };

        (self.update)(message, model_accessor);
    }

    pub async fn view(&self) -> Box<dyn Dom<Event>> {
        Box::new(ComponentDom {
            label: self.label.clone(),
            update_flag: Arc::clone(&self.model_update_flag),
            model_accessor: ModelAccessor {
                model: Arc::clone(&self.model),
                update_flag: Arc::clone(&self.model_update_flag),
            },
            device: self.device,
            event: self.event,
            dom: (self.view)(&*self.model.read().await),
        })
    }
}

// MARK: - ComponentDom

pub struct ComponentDom<Model: Sync + 'static, Event: 'static, InnerEvent: 'static> {
    label: Option<String>,

    update_flag: Arc<UpdateFlag>,
    model_accessor: ModelAccessor<Model>,
    device: fn(&DeviceEvent, ModelAccessor<Model>),
    event: fn(InnerEvent, ModelAccessor<Model>) -> Option<Event>,

    dom: Box<dyn Dom<InnerEvent>>,
}

#[async_trait::async_trait]
impl<Model: Sync + Send + 'static, Event: 'static, InnerEvent: 'static> Dom<Event>
    for ComponentDom<Model, Event, InnerEvent>
{
    // make update_flag to `false` when `build_widget_tree` or `update_widget_tree` called.
    fn build_widget_tree(&self) -> Box<dyn Widget<Event>> {
        self.model_accessor.update_flag.set_to_false();

        Box::new(ComponentWidget {
            label: self.label.clone(),
            model_accessor: self.model_accessor.clone(),
            device: self.device,
            event: self.event,
            widget: self.dom.build_widget_tree(),
        })
    }

    async fn set_update_notifier(&self, notifier: &UpdateNotifier) {
        self.update_flag.set_update_notifier(notifier).await;
        self.dom.set_update_notifier(notifier).await;
    }
}

// MARK: - ComponentWidget

pub struct ComponentWidget<Model: Sync + 'static, Event: 'static, InnerEvent: 'static> {
    label: Option<String>,

    model_accessor: ModelAccessor<Model>,
    device: fn(&DeviceEvent, ModelAccessor<Model>),
    event: fn(InnerEvent, ModelAccessor<Model>) -> Option<Event>,

    widget: Box<dyn Widget<InnerEvent>>,
}

#[async_trait::async_trait]
impl<Model: Sync + Send + 'static, Event: 'static, InnerEvent: 'static> Widget<Event>
    for ComponentWidget<Model, Event, InnerEvent>
{
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    // make update_flag to false when `build_widget_tree` or `update_widget_tree` called.
    async fn update_widget_tree(
        &mut self,
        _: bool,
        dom: &dyn Dom<Event>,
    ) -> Result<(), UpdateWidgetError> {
        // todo: optimize
        if let Some(component_dom) =
            (dom as &dyn Any).downcast_ref::<ComponentDom<Model, Event, InnerEvent>>()
        {
            let is_component_updated = component_dom.update_flag.is_updated();

            self.model_accessor.update_flag.set_to_false();

            // todo: optimize and update label

            self.widget
                .update_widget_tree(is_component_updated, component_dom.dom.as_ref())
                .await
        } else {
            return Err(UpdateWidgetError::TypeMismatch);
        }
    }

    fn compare(&self, dom: &dyn Dom<Event>) -> DomCompareResult {
        // todo: optimize
        if let Some(component_dom) =
            (dom as &dyn Any).downcast_ref::<ComponentDom<Model, Event, InnerEvent>>()
        {
            if self.label == component_dom.label {
                DomCompareResult::Same
            } else {
                DomCompareResult::Different
            }
        } else {
            DomCompareResult::Different
        }
    }

    fn device_event(&mut self, event: &DeviceEvent, context: &WidgetContext) -> Option<Event> {
        (self.device)(event, self.model_accessor.clone());

        self.widget
            .device_event(event, context)
            .and_then(|inner_event| (self.event)(inner_event, self.model_accessor.clone()))
    }

    fn preferred_size(&mut self, constraints: &Constraints, context: &WidgetContext) -> [f32; 2] {
        self.widget.preferred_size(constraints, context)
    }

    fn arrange(&mut self, final_size: [f32; 2], context: &WidgetContext) {
        self.widget.arrange(final_size, context)
    }

    fn is_inside(&mut self, position: [f32; 2], context: &WidgetContext) -> bool {
        self.widget.is_inside(position, context)
    }

    fn need_rerendering(&self) -> bool {
        self.widget.need_rerendering()
    }

    fn render(&mut self, background: Background, context: &WidgetContext) -> RenderNode {
        self.widget.render(background, context)
    }
}
