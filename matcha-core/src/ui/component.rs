use std::{
    any::Any,
    sync::{Arc, atomic::AtomicBool},
};

use tokio::sync::{Mutex, RwLock, RwLockReadGuard};
use utils::back_prop_dirty::BackPropDirty;

use crate::{
    device_input::DeviceInput,
    metrics::Constraints,
    ui::{
        AnyWidget, AnyWidgetFrame, ApplicationHandler, Background, Dom, UpdateWidgetError,
        WidgetContext, widget::AnyWidgetFramePrivate,
    },
    update_flag::UpdateNotifier,
};

type SetupFn<Model> = dyn Fn(&ModelAccessor<Model>, &ApplicationHandler) + Send + Sync;
type UpdateFn<Model, Message> =
    dyn Fn(&Message, &ModelAccessor<Model>, &ApplicationHandler) + Send + Sync;
type InputFn<Model> =
    dyn Fn(&DeviceInput, &ModelAccessor<Model>, &ApplicationHandler) + Send + Sync;
type EventFn<Model, Event, InnerEvent> =
    dyn Fn(InnerEvent, &ModelAccessor<Model>, &ApplicationHandler) -> Option<Event> + Send + Sync;
type ViewFn<Model, InnerEvent> = dyn Fn(&Model) -> Box<dyn Dom<InnerEvent>> + Send + Sync;

fn default_input_function<Model: Send + Sync + 'static>(
    input: &DeviceInput,
    _model_accessor: &ModelAccessor<Model>,
    app_handler: &ApplicationHandler,
) {
    if input.event() == &crate::device_input::DeviceInputData::CloseRequested {
        app_handler.quit();
    }
}

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
    setup: Box<SetupFn<Model>>,
    // update model with message
    update: Box<UpdateFn<Model, Message>>,
    // update model with device event
    input: Arc<InputFn<Model>>,
    // update model with inner event and can emit new event
    event: Arc<EventFn<Model, Event, InnerEvent>>,
    // view function
    view: Box<ViewFn<Model, InnerEvent>>,
}

/// constructor
impl<Model: Send + Sync + 'static, Message, Event: 'static, InnerEvent: 'static>
    Component<Model, Message, Event, InnerEvent>
{
    pub fn new(
        label: Option<&str>,
        model: Model,
        view: impl Fn(&Model) -> Box<dyn Dom<InnerEvent>> + Send + Sync + 'static,
    ) -> Self {
        Self {
            label: label.map(|s| s.to_string()),
            model: Arc::new(RwLock::new(model)),
            model_update_flag: Arc::new(UpdateFlag::new(false)),
            setup: Box::new(|_: &ModelAccessor<Model>, _: &ApplicationHandler| {}),
            update: Box::new(|_: &Message, _: &ModelAccessor<Model>, _: &ApplicationHandler| {}),
            input: Arc::new(default_input_function),
            event: Arc::new(|_: InnerEvent, _: &ModelAccessor<Model>, _: &ApplicationHandler| None),
            view: Box::new(view),
        }
    }

    pub fn setup_fn(
        mut self,
        f: impl Fn(&ModelAccessor<Model>, &ApplicationHandler) + Send + Sync + 'static,
    ) -> Self {
        self.setup = Box::new(f);
        self
    }

    pub fn update_fn(
        mut self,
        f: impl Fn(&Message, &ModelAccessor<Model>, &ApplicationHandler) + Send + Sync + 'static,
    ) -> Self {
        self.update = Box::new(f);
        self
    }

    pub fn input_fn(
        mut self,
        f: impl Fn(&DeviceInput, &ModelAccessor<Model>, &ApplicationHandler) + Send + Sync + 'static,
    ) -> Self {
        self.input = Arc::new(f);
        self
    }

    pub fn event_fn<NewEventType: 'static>(
        self,
        f: impl Fn(InnerEvent, &ModelAccessor<Model>, &ApplicationHandler) -> Option<NewEventType>
        + Send
        + Sync
        + 'static,
    ) -> Component<Model, Message, NewEventType, InnerEvent> {
        Component {
            label: self.label,
            model: self.model,
            model_update_flag: self.model_update_flag,
            setup: self.setup,
            update: self.update,
            input: self.input,
            event: Arc::new(f),
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

    pub fn update(&self, message: &Message, app_handler: &ApplicationHandler) {
        let model_accessor = ModelAccessor {
            model: Arc::clone(&self.model),
            update_flag: Arc::clone(&self.model_update_flag),
        };

        (self.update)(message, &model_accessor, app_handler);
    }

    pub async fn view(&self) -> Box<dyn Dom<Event>> {
        Box::new(ComponentDom {
            label: self.label.clone(),
            model_access: ModelAccessor {
                model: Arc::clone(&self.model),
                update_flag: Arc::clone(&self.model_update_flag),
            },
            input: Arc::clone(&self.input),
            event: Arc::clone(&self.event),
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
    input: Arc<InputFn<Model>>,
    event: Arc<EventFn<Model, Event, InnerEvent>>,

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
            input: Arc::clone(&self.input),
            event: Arc::clone(&self.event),
            widget_tree: self.dom_tree.build_widget_tree(),
        })
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
    input: Arc<InputFn<Model>>,
    event: Arc<EventFn<Model, Event, InnerEvent>>,

    widget_tree: Box<dyn AnyWidgetFrame<InnerEvent>>,
}

impl<Model: Send + Sync + 'static, Event: 'static, InnerEvent: 'static> AnyWidget<Event>
    for ComponentWidget<Model, Event, InnerEvent>
{
    fn device_input(
        &mut self,
        event: &DeviceInput,
        ctx: &WidgetContext,
        app_handler: &ApplicationHandler,
    ) -> Option<Event> {
        (self.input)(event, &self.model_access, app_handler);

        let inner_event = self.widget_tree.device_input(event, ctx, app_handler);
        inner_event.and_then(|e| (self.event)(e, &self.model_access, app_handler))
    }

    fn is_inside(&self, position: [f32; 2], ctx: &WidgetContext) -> bool {
        self.widget_tree.is_inside(position, ctx)
    }

    fn measure(&self, constraints: &Constraints, ctx: &WidgetContext) -> [f32; 2] {
        self.widget_tree.measure(constraints, ctx)
    }

    fn render(
        &self,
        bounds: [f32; 2],
        background: Background,
        ctx: &WidgetContext,
    ) -> renderer::render_node::RenderNode {
        self.widget_tree.render(bounds, background, ctx)
    }
}

impl<Model: Send + Sync + 'static, Event: 'static, InnerEvent: 'static> AnyWidgetFramePrivate
    for ComponentWidget<Model, Event, InnerEvent>
{
    fn arrange(&self, bounds: [f32; 2], ctx: &WidgetContext) {
        self.widget_tree.arrange(bounds, ctx)
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

    async fn set_model_update_notifier(&self, notifier: &UpdateNotifier) {
        self.model_access
            .update_flag
            .set_update_notifier(notifier)
            .await;
        self.widget_tree.set_model_update_notifier(notifier).await;
    }

    fn update_dirty_flags(&mut self, rearrange_flags: BackPropDirty, redraw_flags: BackPropDirty) {
        self.widget_tree
            .update_dirty_flags(rearrange_flags, redraw_flags);
    }

    fn update_gpu_device(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.widget_tree.update_gpu_device(device, queue);
    }
}
