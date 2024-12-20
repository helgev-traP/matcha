use std::sync::{Arc, Mutex, MutexGuard};

use super::{
    context::SharedContext, events::UiEventResult, types::size::{Size, StdSize}, ui::{Dom, DomComPareResult, Widget}, vertex::uv_vertex::UvVertex
};

pub struct Component<Model, Message, OuterResponse, InnerResponse> {
    label: Option<String>,

    model: Arc<Mutex<Model>>,
    model_updated: Arc<Mutex<bool>>,
    fn_update: fn(&ComponentAccess<Model>, Message),
    fn_inner_update:
        fn(&ComponentAccess<Model>, UiEventResult<InnerResponse>) -> UiEventResult<OuterResponse>,
    fn_view: fn(&Model) -> Box<dyn Dom<InnerResponse>>,

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
            model: Arc::new(Mutex::new(model)),
            model_updated: Arc::new(Mutex::new(true)),
            fn_update: update,
            fn_inner_update: |_, _| Default::default(),
            fn_view: view,
            widget_tree: None,
        }
    }

    pub fn inner_update(
        mut self,
        inner_update: fn(
            &ComponentAccess<Model>,
            UiEventResult<InnerResponse>,
        ) -> UiEventResult<OuterResponse>,
    ) -> Self {
        self.fn_inner_update = inner_update;
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
            &ComponentAccess {
                model: self.model.clone(),
                model_updated: self.model_updated.clone(),
            },
            message,
        );

        // todo: make it update widget tree also after inner update.
        // todo: this block may be moved to another place.
        // if *self.model_updated.lock().unwrap() {
        //     self.update_widget_tree();
        //     *self.model_updated.lock().unwrap() = false;
        // }
    }

    fn update_local(
        &mut self,
        event: UiEventResult<InnerResponse>,
    ) -> UiEventResult<OuterResponse> {
        (self.fn_inner_update)(
            &ComponentAccess {
                model: self.model.clone(),
                model_updated: self.model_updated.clone(),
            },
            event,
        )
    }

    fn update_widget_tree(&mut self) {
        let dom = (self.fn_view)(&*self.model.lock().unwrap());

        if let Some(ref mut render_tree) = self.widget_tree {
            if let Ok(_) = render_tree.lock().unwrap().update_widget_tree(&*dom) {
                return;
            }
            *self.widget_tree.as_ref().unwrap().lock().unwrap() = dom.build_widget_tree();
        } else {
            self.widget_tree = Some(Arc::new(Mutex::new(dom.build_widget_tree())));
        }
    }

    pub fn view(&mut self) -> Arc<dyn Dom<OuterResponse>> {
        if self.widget_tree.is_none() || *self.model_updated.lock().unwrap() {
            self.update_widget_tree();

            *self.model_updated.lock().unwrap() = false;
        }

        Arc::new(ComponentDom {
            label: self.label.clone(),
            component_model: ComponentAccess {
                model: self.model.clone(),
                model_updated: self.model_updated.clone(),
            },
            fn_inner_udate: self.fn_inner_update,
            render_tree: self.widget_tree.as_ref().unwrap().clone(),
        })
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

    pub fn model_mut(&self) -> MutexGuard<Model> {
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
    fn_inner_udate:
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
    fn build_widget_tree(&self) -> Box<dyn Widget<OuterResponse>> {
        Box::new(ComponentWidget {
            label: self.label.clone(),
            component_model: self.component_model.clone(),
            local_update_component: self.fn_inner_udate,
            node: self.render_tree.clone(),
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct ComponentWidget<Model, OuterResponse: 'static, InnerResponse: 'static> {
    label: Option<String>,
    component_model: ComponentAccess<Model>,
    local_update_component:
        fn(&ComponentAccess<Model>, UiEventResult<InnerResponse>) -> UiEventResult<OuterResponse>,
    node: Arc<Mutex<Box<dyn Widget<InnerResponse>>>>,
}

impl<Model, O, I> Widget<O> for ComponentWidget<Model, O, I> {
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn widget_event(
        &mut self,
        event: &super::events::UiEvent,
        parent_size: [StdSize; 2],
        context: &SharedContext,
    ) -> UiEventResult<O> {
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

    fn compare(&self, _: &dyn Dom<O>) -> DomComPareResult {
        DomComPareResult::Different
    }

    fn update_widget_tree(&mut self, _: &dyn Dom<O>) -> Result<(), ()> {
        Ok(())
    }

    fn size(&self) -> [Size; 2] {
        self.node.lock().unwrap().size()
    }

    fn px_size(&self, parent_size: [StdSize; 2], context: &SharedContext) -> [f32; 2] {
        self.node.lock().unwrap().px_size(parent_size, context)
    }

    fn default_size(&self) -> [f32; 2] {
        self.node.lock().unwrap().default_size()
    }

    fn render(
        &mut self,
        // ui environment
        parent_size: [StdSize; 2],
        // context
        context: &SharedContext,
        renderer: &super::renderer::Renderer,
        frame: u64,
    ) -> Vec<(
        Arc<wgpu::Texture>,
        Arc<Vec<UvVertex>>,
        Arc<Vec<u16>>,
        nalgebra::Matrix4<f32>,
    )> {
        self.node
            .lock()
            .unwrap()
            .render(parent_size, context, renderer, frame)
    }
}
