use std::any::{Any, TypeId};

use fxhash::FxHashMap;

use super::context::WidgetContext;

pub mod principle_renderer;

// MARK: RendererMap

#[derive(Default)]
pub struct RendererMap {
    set: FxHashMap<TypeId, Box<dyn Renderer>>,
}

impl RendererMap {
    pub fn new() -> Self {
        Self {
            set: FxHashMap::default(),
        }
    }

    pub fn add_only<T: Renderer>(&mut self, renderer: T) {
        self.set.insert(TypeId::of::<T>(), Box::new(renderer));
    }

    pub fn add<T: Renderer>(&mut self, ctx: &WidgetContext, mut renderer: T) {
        let device = ctx.device();
        let queue = ctx.queue();
        let format = ctx.texture_format();
        renderer.setup(device, queue, format);

        self.set.insert(TypeId::of::<T>(), Box::new(renderer));
    }

    pub(crate) fn setup(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) {
        for renderer in self.set.values_mut() {
            renderer.setup(device, queue, format);
        }
    }

    pub fn get<T: Renderer>(&self) -> Option<&T> {
        self.set
            .get(&TypeId::of::<T>())
            .and_then(|renderer| (renderer.as_ref() as &dyn Any).downcast_ref::<T>())
    }
}

// MARK: Renderer

pub trait Renderer: Any + Send {
    fn setup(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat); // todo: add some error handling
}
