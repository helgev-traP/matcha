use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};

use super::{
    context::SharedContext,
    events::UiEventResult,
    types::range::Range2D,
    ui::{Dom, DomComPareResult, Object, UiBackground, UiContext, UpdateWidgetError, Widget},
};

// MARK: - Component

pub struct Component<Model, Message, OuterResponse, InnerResponse> {
    label: Option<String>,

    // model
    // shared with ComponentAccess
    model: Arc<RwLock<Model>>,
    model_updated: Arc<RwLock<bool>>,

    // update function
    // update from the outside Message
    update_fn: fn(&ComponentAccess<Model>, Message),
    // update from the inside InnerResponse and return OuterResponse
    react_fn:
        fn(&ComponentAccess<Model>, UiEventResult<InnerResponse>) -> UiEventResult<OuterResponse>,

    // view function
    fn_view: fn(&Model) -> Box<dyn Dom<InnerResponse>>,

    // cached widget tree
    widget_tree: Option<Arc<Mutex<Box<dyn Widget<InnerResponse>>>>>,
}

// MARK: - constructor

// Component constructor
impl<Model: Send + 'static, Message, OuterResponse: 'static, InnerResponse: 'static>
    Component<Model, Message, OuterResponse, InnerResponse>
{
    pub fn new(
        label: Option<String>,
        model: Model,
        update: fn(&ComponentAccess<Model>, Message),
        view: fn(&Model) -> Box<dyn Dom<InnerResponse>>,
    ) -> Self {
        Self {
            label,
            model: Arc::new(RwLock::new(model)),
            model_updated: Arc::new(RwLock::new(true)),
            update_fn: update,
            react_fn: |_, _| Default::default(),
            fn_view: view,
            widget_tree: None,
        }
    }

    // react to the inner response and return the outer response
    pub fn react_fn(
        mut self,
        react: fn(
            &ComponentAccess<Model>,
            UiEventResult<InnerResponse>,
        ) -> UiEventResult<OuterResponse>,
    ) -> Self {
        self.react_fn = react;
        self
    }
}

// MARK: - methods

// access to the model immutable
// todo: モデルの変更を直接行うか、メッセージを通して行うか判断。
impl<Model: Send + 'static, Message, OuterResponse: 'static, InnerResponse: 'static>
    Component<Model, Message, OuterResponse, InnerResponse>
{
    pub fn model_ref(&self) -> RwLockReadGuard<Model> {
        self.model.read().unwrap()
    }
}

// methods
impl<Model: Send + 'static, Message, OuterResponse: 'static, InnerResponse: 'static>
    Component<Model, Message, OuterResponse, InnerResponse>
{
    pub fn label(&self) -> Option<&String> {
        self.label.as_ref()
    }

    pub fn update(&mut self, message: Message) {
        (self.update_fn)(
            &ComponentAccess {
                model: self.model.clone(),
                model_updated: self.model_updated.clone(),
            },
            message,
        );
    }

    pub fn view(&mut self) -> Arc<dyn Dom<OuterResponse>> {
        if self.widget_tree.is_none() || *self.model_updated.read().unwrap() {
            self.update_widget_tree();

            *self.model_updated.write().unwrap() = false;
        }

        Arc::new(ComponentDom {
            label: self.label.clone(),
            component_model: ComponentAccess {
                model: self.model.clone(),
                model_updated: self.model_updated.clone(),
            },
            react_fn: self.react_fn,
            widget_tree: self.widget_tree.as_ref().unwrap().clone(),
        })
    }
}

// MARK: - inner methods

// inner methods
impl<Model: Send + 'static, Message, OuterResponse: 'static, InnerResponse: 'static>
    Component<Model, Message, OuterResponse, InnerResponse>
{
    // todo: this seems not necessary
    // fn update_local(
    //     &mut self,
    //     event: UiEventResult<InnerResponse>,
    // ) -> UiEventResult<OuterResponse> {
    //     (self.react_fn)(
    //         &ComponentAccess {
    //             model: self.model.clone(),
    //             model_updated: self.model_updated.clone(),
    //         },
    //         event,
    //     )
    // }

    fn update_widget_tree(&mut self) {
        let dom = (self.fn_view)(&*self.model.read().unwrap());

        if let Some(ref mut render_tree) = self.widget_tree {
            if render_tree
                .lock()
                .unwrap()
                .update_widget_tree(&*dom)
                .is_ok()
            {
                return;
            }
            *self.widget_tree.as_ref().unwrap().lock().unwrap() = dom.build_widget_tree();
        } else {
            self.widget_tree = Some(Arc::new(Mutex::new(dom.build_widget_tree())));
        }
    }
}

// MARK: - ComponentAccess

pub struct ComponentAccess<Model> {
    model: Arc<RwLock<Model>>,
    model_updated: Arc<RwLock<bool>>,
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
    pub fn model_ref(&self) -> RwLockReadGuard<Model> {
        self.model.read().unwrap()
    }

    pub fn model_mut(&self) -> RwLockWriteGuard<Model> {
        *self.model_updated.write().unwrap() = true;
        self.model.write().unwrap()
    }
}

// MARK: - ComponentDom

pub struct ComponentDom<Model, OuterResponse, InnerResponse>
where
    Model: Send + 'static,
    OuterResponse: 'static,
    InnerResponse: 'static,
{
    label: Option<String>,
    component_model: ComponentAccess<Model>,
    react_fn:
        fn(&ComponentAccess<Model>, UiEventResult<InnerResponse>) -> UiEventResult<OuterResponse>,
    widget_tree: Arc<Mutex<Box<dyn Widget<InnerResponse>>>>,
}

impl<Model: Send, OuterResponse, InnerResponse> Dom<OuterResponse>
    for ComponentDom<Model, OuterResponse, InnerResponse>
where
    Model: 'static,
    OuterResponse: 'static,
    InnerResponse: 'static,
{
    fn build_widget_tree(&self) -> Box<dyn Widget<OuterResponse>> {
        Box::new(ComponentWidget {
            label: self.label.clone(),
            component_model: self.component_model.clone(),
            local_update_component: self.react_fn,
            node: self.widget_tree.clone(),
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// MARK: - ComponentWidget

pub struct ComponentWidget<Model, OuterResponse, InnerResponse>
where
    Model: 'static,
    OuterResponse: 'static,
    InnerResponse: 'static,
{
    label: Option<String>,
    component_model: ComponentAccess<Model>,
    local_update_component:
        fn(&ComponentAccess<Model>, UiEventResult<InnerResponse>) -> UiEventResult<OuterResponse>,
    node: Arc<Mutex<Box<dyn Widget<InnerResponse>>>>,
}

impl<Model, OuterResponse, InnerResponse> Widget<OuterResponse>
    for ComponentWidget<Model, OuterResponse, InnerResponse>
{
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn update_widget_tree(
        &mut self,
        dom: &dyn Dom<OuterResponse>,
    ) -> Result<(), UpdateWidgetError> {
        if dom.as_any().is::<Self>() {
            Ok(())
        } else {
            Err(UpdateWidgetError::TypeMismatch)
        }
    }

    fn compare(&self, _: &dyn Dom<OuterResponse>) -> DomComPareResult {
        DomComPareResult::Different
    }

    fn widget_event(
        &mut self,
        event: &super::events::UiEvent,
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
        tag: u64,
        frame: u64,
    ) -> UiEventResult<OuterResponse> {
        (self.local_update_component)(
            &self.component_model,
            self.node
                .lock()
                .unwrap()
                .widget_event(event, parent_size, context, tag, frame),
        )
    }

    fn is_inside(
        &mut self,
        position: [f32; 2],
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
        tag: u64,
        frame: u64,
    ) -> bool {
        self.node
            .lock()
            .unwrap()
            .is_inside(position, parent_size, context, tag, frame)
    }

    fn px_size(
        &mut self,
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
        tag: u64,
        frame: u64,
    ) -> [f32; 2] {
        self.node
            .lock()
            .unwrap()
            .px_size(parent_size, context, tag, frame)
    }

    fn draw_range(
        &mut self,
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
        tag: u64,
        frame: u64,
    ) -> Option<Range2D<f32>> {
        self.node
            .lock()
            .unwrap()
            .draw_range(parent_size, context, tag, frame)
    }

    fn cover_area(
        &mut self,
        parent_size: [Option<f32>; 2],
        context: &SharedContext,
        tag: u64,
        frame: u64,
    ) -> Option<Range2D<f32>> {
        self.node
            .lock()
            .unwrap()
            .cover_area(parent_size, context, tag, frame)
    }

    fn redraw(&self) -> bool {
        self.node.lock().unwrap().redraw()
    }

    fn render(&mut self, ui_background: UiBackground, ui_context: UiContext) -> Vec<Object> {
        self.node.lock().unwrap().render(ui_background, ui_context)
    }
}
