use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};

use super::{
    context::SharedContext,
    events::UiEventResult,
    types::{
        range::Range2D,
        size::{Size, StdSize},
    },
    ui::{Dom, DomComPareResult, Object, Widget},
};

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

// access to the model immutable
// todo: モデルの変更を直接行うか、メッセージを通して行うか判断。
impl<Model: Send + 'static, Message, OuterResponse: 'static, InnerResponse: 'static>
    Component<Model, Message, OuterResponse, InnerResponse>
{
    pub fn model_ref(&self) -> RwLockReadGuard<Model> {
        self.model.read().unwrap()
    }
}

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

    fn update_local(
        &mut self,
        event: UiEventResult<InnerResponse>,
    ) -> UiEventResult<OuterResponse> {
        (self.react_fn)(
            &ComponentAccess {
                model: self.model.clone(),
                model_updated: self.model_updated.clone(),
            },
            event,
        )
    }

    fn update_widget_tree(&mut self) {
        let dom = (self.fn_view)(&*self.model.read().unwrap());

        if let Some(ref mut render_tree) = self.widget_tree {
            if let Ok(_) = render_tree.lock().unwrap().update_widget_tree(&*dom) {
                return;
            }
            *self.widget_tree.as_ref().unwrap().lock().unwrap() = dom.build_widget_tree().0;
        } else {
            self.widget_tree = Some(Arc::new(Mutex::new(dom.build_widget_tree().0)));
        }
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

// impl<Model: Send, OuterResponse, InnerResponse> ComponentDom<Model, OuterResponse, InnerResponse>
// where
//     Model: 'static,
//     OuterResponse: 'static,
//     InnerResponse: 'static,
// {
//     fn build_widget_tree_as_root(
//         &self,
//         background_color: [f32; 4],
//     ) -> Box<dyn Widget<OuterResponse>> {
//         Box::new(ComponentWidget {
//             label: self.label.clone(),
//             component_model: self.component_model.clone(),
//             local_update_component: self.react_fn,
//             node: self.widget_tree.clone(),
//             root_background_color: Some(background_color),
//             root_texture: None,
//         })
//     }
// }

impl<Model: Send, OuterResponse, InnerResponse> Dom<OuterResponse>
    for ComponentDom<Model, OuterResponse, InnerResponse>
where
    Model: 'static,
    OuterResponse: 'static,
    InnerResponse: 'static,
{
    fn build_widget_tree(&self) -> (Box<dyn Widget<OuterResponse>>, bool) {
        (
            Box::new(ComponentWidget {
                label: self.label.clone(),
                component_model: self.component_model.clone(),
                local_update_component: self.react_fn,
                node: self.widget_tree.clone(),
                root_background_color: None,
                root_texture: None,
            }),
            self.widget_tree.lock().unwrap().has_dynamic(),
        )
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

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
    // if this is root component
    root_background_color: Option<[f32; 4]>,
    root_texture: Option<wgpu::Texture>,
}

impl<Model, OuterResponse, InnerResponse> ComponentWidget<Model, OuterResponse, InnerResponse>
where
    Model: 'static,
    OuterResponse: 'static,
    InnerResponse: 'static,
{
    pub(crate) fn render_as_root(
        &mut self,
        // ui environment
        parent_size: [StdSize; 2],
        // context
        context: &SharedContext,
        renderer: &super::renderer::Renderer,
        frame: u64,
    ) -> Vec<Object> {
        let texture = self.root_texture.get_or_insert_with(|| {
            // prepare the background texture

            todo!()
        });

        let background_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let background_range = Range2D {
            x: [0.0, 1.0],
            y: [0.0, 1.0],
        };

        self.node.lock().unwrap().render(
            parent_size,
            &background_view,
            background_range,
            context,
            renderer,
            frame,
        )
    }
}

impl<Model, OuterResponse, InnerResponse> Widget<OuterResponse>
    for ComponentWidget<Model, OuterResponse, InnerResponse>
{
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn widget_event(
        &mut self,
        event: &super::events::UiEvent,
        parent_size: [StdSize; 2],
        context: &SharedContext,
    ) -> UiEventResult<OuterResponse> {
        (self.local_update_component)(
            &self.component_model,
            self.node
                .lock()
                .unwrap()
                .widget_event(event, parent_size, context),
        )
    }

    fn is_inside(
        &self,
        position: [f32; 2],
        parent_size: [StdSize; 2],
        context: &SharedContext,
    ) -> bool {
        self.node
            .lock()
            .unwrap()
            .is_inside(position, parent_size, context)
    }

    fn compare(&self, _: &dyn Dom<OuterResponse>) -> DomComPareResult {
        DomComPareResult::Different
    }

    fn update_widget_tree(&mut self, _: &dyn Dom<OuterResponse>) -> Result<(), ()> {
        Ok(())
    }

    fn size(&self) -> [Size; 2] {
        self.node.lock().unwrap().size()
    }

    fn px_size(&self, parent_size: [StdSize; 2], context: &SharedContext) -> [f32; 2] {
        self.node.lock().unwrap().px_size(parent_size, context)
    }

    fn drawing_range(&self, parent_size: [StdSize; 2], context: &SharedContext) -> [[f32; 2]; 2] {
        self.node
            .lock()
            .unwrap()
            .drawing_range(parent_size, context)
    }

    fn cover_area(
        &self,
        parent_size: [StdSize; 2],
        context: &SharedContext,
    ) -> Option<[[f32; 2]; 2]> {
        self.node.lock().unwrap().cover_area(parent_size, context)
    }

    fn has_dynamic(&self) -> bool {
        self.node.lock().unwrap().has_dynamic()
    }

    fn redraw(&self) -> bool {
        self.node.lock().unwrap().redraw()
    }

    fn render(
        &mut self,
        // ui environment
        parent_size: [StdSize; 2],
        background_view: &wgpu::TextureView,
        background_range: Range2D<f32>,
        // context
        context: &SharedContext,
        renderer: &super::renderer::Renderer,
        frame: u64,
    ) -> Vec<Object> {
        self.node.lock().unwrap().render(
            parent_size,
            background_view,
            background_range,
            context,
            renderer,
            frame,
        )
    }
}
