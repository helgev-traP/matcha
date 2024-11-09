use std::{
    cell::RefCell,
    collections::HashMap,
    hash::Hash,
    rc::Rc,
    sync::{Arc, Mutex, MutexGuard},
};

use vello::{
    kurbo,
    wgpu::{self, Texture},
    Scene,
};

use super::{
    context::SharedContext,
    events::UiEventResult,
    types::size::PxSize,
    ui::{Dom, DomComPareResult, LayerStack, TextureLayer, Widget},
};

// --- Main body of the UI component ---

pub struct Component<Model, Message, OuterResponse, InnerResponse> {
    label: Option<String>,

    // model
    model: Arc<Mutex<Model>>,
    model_updated: Arc<Mutex<bool>>,

    // update model from global event.
    update_fn: fn(ComponentAccess<Model>, Message),

    // update model from local event.
    local_update_fn:
        fn(&ComponentAccess<Model>, UiEventResult<InnerResponse>) -> UiEventResult<OuterResponse>,

    // view
    view_fn: fn(&Model) -> Box<dyn Dom<InnerResponse>>,

    // stored render tree
    render_tree: Option<Rc<RefCell<Box<dyn Widget<InnerResponse>>>>>,
}

// initialization

impl<Model: Send + 'static, Message, OuterResponse: 'static, InnerResponse: 'static>
    Component<Model, Message, OuterResponse, InnerResponse>
{
    pub fn new(
        label: Option<&str>,
        model: Model,
        update: fn(ComponentAccess<Model>, Message),
        view: fn(&Model) -> Box<dyn Dom<InnerResponse>>,
    ) -> Self {
        Self {
            label: label.map(|s| s.to_string()),
            model: Arc::new(Mutex::new(model)),
            model_updated: Arc::new(Mutex::new(true)),
            update_fn: update,
            local_update_fn: |_, _| Default::default(),
            view_fn: view,
            render_tree: None,
        }
    }

    pub fn local_update_fn(
        mut self,
        local_update_fn: fn(
            &ComponentAccess<Model>,
            UiEventResult<InnerResponse>,
        ) -> UiEventResult<OuterResponse>,
    ) -> Self {
        self.local_update_fn = local_update_fn;
        self
    }
}

// assume to be called in view function / update function of superior component.

impl<Model: Send + 'static, Message, OuterResponse: 'static, InnerResponse: 'static>
    Component<Model, Message, OuterResponse, InnerResponse>
{
    pub fn label(&self) -> Option<&String> {
        self.label.as_ref()
    }

    pub fn update(&mut self, message: Message) {
        (self.update_fn)(
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

    pub fn view(&mut self) -> Box<dyn Dom<OuterResponse>> {
        if let None = self.render_tree {
            self.update_render_tree();
        }
        Box::new(ComponentDom {
            label: self.label.clone(),
            component_model: ComponentAccess {
                model: self.model.clone(),
                model_updated: self.model_updated.clone(),
            },
            local_update_fn: self.local_update_fn,
            render_tree: self.render_tree.as_ref().unwrap().clone(),
        })
    }

    pub(crate) fn component_render_node(
        &mut self,
    ) -> ComponentRenderNode<Model, OuterResponse, InnerResponse> {
        if let None = self.render_tree {
            self.update_render_tree();
        }
        ComponentRenderNode {
            label: self.label.clone(),
            component_access: ComponentAccess {
                model: self.model.clone(),
                model_updated: self.model_updated.clone(),
            },
            local_update_fn: self.local_update_fn,
            render_tree: self.render_tree.as_ref().unwrap().clone(),
            texture_cache: TextureDualCache::new(),
        }
    }

    fn update_render_tree(&mut self) {
        let dom = (self.view_fn)(&*self.model.lock().unwrap());

        if let Some(ref mut render_tree) = self.render_tree {
            if let Ok(_) = render_tree.borrow_mut().update_render_tree(&*dom) {
                return;
            }
            self.render_tree = Some(Rc::new(RefCell::new(dom.build_render_tree())));
        } else {
            self.render_tree = Some(Rc::new(RefCell::new(dom.build_render_tree())));
        }
    }
}

// --- ComponentAccess ---
// use this to check if the model is changed.

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
    pub fn model_ref<'a>(&'a self) -> MutexGuard<'a, Model> {
        self.model.lock().unwrap()
    }

    pub fn model_mut<'a>(&'a mut self) -> MutexGuard<'a, Model> {
        *self.model_updated.lock().unwrap() = true;
        self.model.lock().unwrap()
    }
}

// --- ComponentDom ---
// Returns by Component::view()
// Component as a Dom.

pub struct ComponentDom<Model, OuterResponse, InnerResponse>
where
    Model: Send + 'static,
    OuterResponse: 'static,
    InnerResponse: 'static,
{
    label: Option<String>,
    component_model: ComponentAccess<Model>,
    local_update_fn:
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
            component_access: self.component_model.clone(),
            local_update_fn: self.local_update_fn,
            render_tree: self.render_tree.clone(),
            texture_cache: TextureDualCache::new(),
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// --- ComponentRenderNode ---
// Returns by ComponentDom::build_render_tree()
// Component as a widget.
// All method of Widget will be delegated to the inner widget except Widget::render().

pub struct ComponentRenderNode<Model, OuterResponse: 'static, InnerResponse: 'static> {
    label: Option<String>,
    component_access: ComponentAccess<Model>,
    local_update_fn:
        fn(&ComponentAccess<Model>, UiEventResult<InnerResponse>) -> UiEventResult<OuterResponse>,
    render_tree: Rc<RefCell<Box<dyn Widget<InnerResponse>>>>,
    texture_cache: TextureDualCache,
}

impl<Model, O, I> Widget<O> for ComponentRenderNode<Model, O, I> {
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn event(
        &mut self,
        event: &super::events::UiEvent,
        parent_size: PxSize,
        context: &SharedContext,
    ) -> UiEventResult<O> {
        (self.local_update_fn)(
            &self.component_access,
            self.render_tree
                .borrow_mut()
                .event(event, parent_size, context),
        )
    }

    fn is_inside(&self, position: [f32; 2], parent_size: PxSize, context: &SharedContext) -> bool {
        self.render_tree
            .borrow()
            .is_inside(position, parent_size, context)
    }

    fn compare(&self, _: &dyn Dom<O>) -> DomComPareResult {
        DomComPareResult::Different
    }

    fn update_render_tree(&mut self, _: &dyn Dom<O>) -> Result<(), ()> {
        Ok(())
    }

    fn size(&self) -> super::types::size::Size {
        self.render_tree.borrow().size()
    }

    fn px_size(&self, parent_size: PxSize, context: &SharedContext) -> PxSize {
        self.render_tree.borrow().px_size(parent_size, context)
    }

    fn default_size(&self) -> super::types::size::PxSize {
        self.render_tree.borrow().default_size()
    }

    fn render(
        &mut self,
        _: Option<&mut Scene>,
        texture_layers: &mut LayerStack,
        parent_size: PxSize,
        affine: vello::kurbo::Affine,
        context: &SharedContext,
    ) {


        // below is the old implementation.
        // // 1. render component's widget to local texture. / collect subordinates' TextureLayers.

        // // prepare texture, texture_stack

        // let size = context.get_viewport_size();

        // let texture = self.texture.get_or_insert_with(|| {
        //     // make texture
        //     Arc::new(
        //         context
        //             .get_device()
        //             .create_texture(&wgpu::TextureDescriptor {
        //                 label: Some("Component Texture"),
        //                 size: wgpu::Extent3d {
        //                     width: size.0,
        //                     height: size.1,
        //                     depth_or_array_layers: 1,
        //                 },
        //                 mip_level_count: 1,
        //                 sample_count: 1,
        //                 dimension: wgpu::TextureDimension::D2,
        //                 format: wgpu::TextureFormat::Rgba8Unorm,
        //                 usage: wgpu::TextureUsages::TEXTURE_BINDING
        //                     | wgpu::TextureUsages::STORAGE_BINDING,
        //                 view_formats: &[],
        //             }),
        //     )
        // });

        // let mut local_texture_stack = LayerStack::new();

        // // let widget render to scene and collect textures to texture_stack.

        // let mut scene = Scene::new();

        // self.render_tree.borrow_mut().render(
        //     &mut scene,
        //     &mut local_texture_stack,
        //     parent_size,
        //     affine,
        //     context,
        // );

        // // render local scene to texture.

        // context
        //     .get_vello_renderer()
        //     .render_to_texture(
        //         context.get_device(),
        //         context.get_queue(),
        //         &mut scene,
        //         &texture.create_view(&wgpu::TextureViewDescriptor::default()),
        //         &vello::RenderParams {
        //             base_color: vello::peniko::Color::TRANSPARENT,
        //             width: size.0,
        //             height: size.1,
        //             antialiasing_method: vello::AaConfig::Msaa8,
        //         },
        //     )
        //     .unwrap();

        // // 2. append texture_layers.

        // texture_layers.push(TextureLayer {
        //     texture: texture.clone(),
        //     position: [0.0, 0.0],
        //     size: [size.0 as f32, size.1 as f32],
        // });

        // texture_layers.extend(local_texture_stack.vec());
    }
}

struct TextureDualCache {
    buffer: [HashMap<PositionSize, Arc<Texture>>; 2],
    current_frame: u64,
}

impl TextureDualCache {
    fn new() -> Self {
        Self {
            buffer: [HashMap::new(), HashMap::new()],
            current_frame: 0,
        }
    }

    fn get_or_insert_with(
        &mut self,
        frame: u64,
        key: &(PxSize, kurbo::Affine),
        f: impl FnOnce() -> Arc<Texture>,
    ) -> Arc<Texture> {
        if frame == self.current_frame {
            // frame is the same as the previous one.


            todo!()
        } else {
            // frame improved.
            // first clear the buffer of the 2 frames ago and update the current frame.
            todo!()
        }

        // below is the old implementation.

        // let key = PositionSize::from(*key);
        // let current = self.current & 1;
        // if let Some(texture) = self.buffer[current].get(&key) {
        //     texture.clone()
        // } else {
        //     let texture = f();
        //     self.buffer[current].insert(key, texture.clone());
        //     texture
        // }
    }
}

#[derive(Hash, Eq, PartialEq)]
struct PositionSize {
    size: [u32; 2],
    position: [u64; 6],
}

impl From<(PxSize, kurbo::Affine)> for PositionSize {
    fn from((size, position): (PxSize, kurbo::Affine)) -> Self {
        let coeffs = position.as_coeffs();
        Self {
            size: [size.width.to_bits(), size.height.to_bits()],
            position: [
                coeffs[0].to_bits(),
                coeffs[1].to_bits(),
                coeffs[2].to_bits(),
                coeffs[3].to_bits(),
                coeffs[4].to_bits(),
                coeffs[5].to_bits(),
            ],
        }
    }
}
