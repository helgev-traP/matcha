use std::{
    any::Any,
    sync::{Arc, atomic::AtomicBool},
};

use tokio::sync::{Mutex, RwLock, RwLockReadGuard};
use utils::back_prop_dirty::BackPropDirty;

use crate::{
    device_input::DeviceInput,
    ui::{
        AnyWidget, AnyWidgetFrame, Background, Constraints, Dom, UpdateWidgetError, WidgetContext,
        widget::AnyWidgetFramePrivate,
    },
    update_flag::UpdateNotifier,
};

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
    setup: fn(&ModelAccessor<Model>),
    // update model with message
    update: fn(&Message, &ModelAccessor<Model>),
    // update model with device event
    input: fn(&DeviceInput, &ModelAccessor<Model>),
    // update model with inner event and can emit new event
    event: fn(InnerEvent, &ModelAccessor<Model>) -> Option<Event>,
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
            setup: |_: &ModelAccessor<Model>| {},
            update: |_: &Message, _: &ModelAccessor<Model>| {},
            input: |_: &DeviceInput, _: &ModelAccessor<Model>| {},
            event: |_: InnerEvent, _: &ModelAccessor<Model>| None,
            view,
        }
    }

    pub fn setup_fn(mut self, f: fn(&ModelAccessor<Model>)) -> Self {
        self.setup = f;
        self
    }

    pub fn update_fn(mut self, f: fn(&Message, &ModelAccessor<Model>)) -> Self {
        self.update = f;
        self
    }

    pub fn input_fn(mut self, f: fn(&DeviceInput, &ModelAccessor<Model>)) -> Self {
        self.input = f;
        self
    }

    pub fn event_fn<NewEventType: 'static>(
        self,
        f: fn(InnerEvent, &ModelAccessor<Model>) -> Option<NewEventType>,
    ) -> Component<Model, Message, NewEventType, InnerEvent> {
        Component {
            label: self.label,
            model: self.model,
            model_update_flag: self.model_update_flag,
            setup: self.setup,
            update: self.update,
            input: self.input,
            event: f,
            view: self.view,
        }
    }
}

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

        (self.update)(message, &model_accessor);
    }

    pub async fn view(&self) -> Box<dyn Dom<Event>> {
        Box::new(ComponentDom {
            label: self.label.clone(),
            model_access: ModelAccessor {
                model: Arc::clone(&self.model),
                update_flag: Arc::clone(&self.model_update_flag),
            },
            input: self.input,
            event: self.event,
            dom_tree: (self.view)(&*self.model.read().await),
        })
    }
}

/// Access point to component model and manage update flag
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

    pub async fn read<F, V>(&self, f: F) -> V
    where
        F: FnOnce(&Model) -> V,
    {
        let model = self.model.read().await;
        f(&*model)
    }

    pub async fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut Model),
    {
        // ensure update function finish before change the update flag
        let mut model = self.model.write().await;
        f(&mut model);
        self.update_flag.set_to_true().await;
    }
}

/// manage component update state and `UpdateNotifier`
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

    // fn set_to_false(&self) {
    //     self.updated
    //         .store(false, std::sync::atomic::Ordering::Release);
    // }

    /// Create an observer receiver. Also reset the update flag to false
    async fn set_update_notifier(&self, notifier: &UpdateNotifier) {
        let mut observer_sender = self.observer_sender.lock().await;
        *observer_sender = Some(notifier.clone());
    }

    // fn is_updated(&self) -> bool {
    //     self.updated.load(std::sync::atomic::Ordering::Acquire)
    // }
}

// MARK: - ComponentDom

pub struct ComponentDom<Model: Send + Sync + 'static, Event: 'static, InnerEvent: 'static = Event> {
    label: Option<String>,

    model_access: ModelAccessor<Model>,
    input: fn(&DeviceInput, &ModelAccessor<Model>),
    event: fn(InnerEvent, &ModelAccessor<Model>) -> Option<Event>,

    dom_tree: Box<dyn Dom<InnerEvent>>,
}

#[async_trait::async_trait]
impl<Model: Send + Sync + 'static, Event: 'static, InnerEvent: 'static> Dom<Event>
    for ComponentDom<Model, Event, InnerEvent>
{
    fn build_widget_tree(&self) -> Box<dyn AnyWidgetFrame<Event>> {
        Box::new(ComponentWidget {
            label: self.label.clone(),
            model_access: self.model_access.clone(),
            input: self.input,
            event: self.event,
            widget_tree: self.dom_tree.build_widget_tree(),
        })
    }

    async fn set_update_notifier(&self, notifier: &UpdateNotifier) {
        self.model_access
            .update_flag
            .set_update_notifier(notifier)
            .await;
        self.dom_tree.set_update_notifier(notifier).await;
    }
}

impl<Model: Send + Sync + 'static, Event: 'static, InnerEvent: 'static>
    ComponentDom<Model, Event, InnerEvent>
{
    fn child_widget(&self) -> &dyn Dom<InnerEvent> {
        &*self.dom_tree
    }
}

// MARK: - ComponentWidget

pub struct ComponentWidget<
    Model: Send + Sync + 'static,
    Event: 'static,
    InnerEvent: 'static = Event,
> {
    label: Option<String>,

    model_access: ModelAccessor<Model>,
    input: fn(&DeviceInput, &ModelAccessor<Model>),
    event: fn(InnerEvent, &ModelAccessor<Model>) -> Option<Event>,

    widget_tree: Box<dyn AnyWidgetFrame<InnerEvent>>,
}

impl<Model: Send + Sync + 'static, Event: 'static, InnerEvent: 'static> AnyWidget<Event>
    for ComponentWidget<Model, Event, InnerEvent>
{
    fn device_event(&mut self, event: &DeviceInput, ctx: &WidgetContext) -> Option<Event> {
        (self.input)(event, &self.model_access);

        let inner_event = self.widget_tree.device_event(event, ctx);
        inner_event.and_then(|e| (self.event)(e, &self.model_access))
    }

    fn is_inside(&self, position: [f32; 2], ctx: &WidgetContext) -> bool {
        self.widget_tree.is_inside(position, ctx)
    }

    fn measure(&self, constraints: &Constraints, ctx: &WidgetContext) -> [f32; 2] {
        self.widget_tree.measure(constraints, ctx)
    }

    fn render(
        &self,
        final_size: [f32; 2],
        background: Background,
        ctx: &WidgetContext,
    ) -> renderer::render_node::RenderNode {
        self.widget_tree.render(final_size, background, ctx)
    }
}

impl<Model: Send + Sync + 'static, Event: 'static, InnerEvent: 'static> AnyWidgetFramePrivate
    for ComponentWidget<Model, Event, InnerEvent>
{
    fn arrange(&self, final_size: [f32; 2], ctx: &WidgetContext) {
        self.widget_tree.arrange(final_size, ctx)
    }
}

#[async_trait::async_trait]
impl<Model: Send + Sync + 'static, Event: 'static, InnerEvent: 'static> AnyWidgetFrame<Event>
    for ComponentWidget<Model, Event, InnerEvent>
{
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn need_redraw(&self) -> bool {
        self.widget_tree.need_redraw()
    }

    async fn update_widget_tree(&mut self, dom: &dyn Dom<Event>) -> Result<(), UpdateWidgetError> {
        let dom = (dom as &dyn Any)
            .downcast_ref::<ComponentDom<Model, Event, InnerEvent>>()
            .ok_or(UpdateWidgetError::TypeMismatch)?;

        let child_widget = dom.child_widget();
        if let Err(UpdateWidgetError::TypeMismatch) =
            self.widget_tree.update_widget_tree(child_widget).await
        {
            // rebuild widget tree
            self.widget_tree = child_widget.build_widget_tree();
        }
        Ok(())
    }

    fn update_dirty_flags(&mut self, rearrange_flags: BackPropDirty, redraw_flags: BackPropDirty) {
        self.widget_tree
            .update_dirty_flags(rearrange_flags, redraw_flags);
    }

    fn update_gpu_device(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.widget_tree.update_gpu_device(device, queue);
    }
}
