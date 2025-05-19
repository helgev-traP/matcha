use std::{
    any::{Any, TypeId},
    sync::{Arc},
};

use fxhash::FxHashMap;
use parking_lot::Mutex;

use super::context::WidgetContext;

pub mod principle_renderer;

// MARK: RendererMap

#[derive(Default)]
pub struct RendererMap {
    set: Mutex<FxHashMap<TypeId, Arc<dyn RendererSetup>>>,
}

impl RendererMap {
    pub fn new() -> Self {
        Self {
            set: Mutex::new(FxHashMap::default()),
        }
    }

    // pub async fn add<T: RendererSetup>(&mut self, ctx: &WidgetContext<'_>, mut renderer: T) {
    //     let device = ctx.device();
    //     let queue = ctx.queue();
    //     let format = ctx.texture_format();
    //     renderer.setup(device, queue, format);

    //     self.set
    //         .write()
    //         .await
    //         .insert(TypeId::of::<T>(), Arc::new(renderer));
    // }

    pub fn get_or_setup<T>(&self, ctx: &WidgetContext<'_>) -> Arc<T>
    where
        T: RendererSetup + Default,
    {
        // Early return if already exists
        if let Some(renderer) = self.set.lock().get(&TypeId::of::<T>()) {
            let renderer = Arc::clone(renderer);
            let arc_any = renderer as Arc<dyn Any + Send + Sync>;

            return arc_any.downcast().unwrap();
        }

        // If not, create a new one
        let device = ctx.device();
        let queue = ctx.queue();
        let format = ctx.texture_format();
        let mut renderer = T::default();
        renderer.setup(device, queue, format);

        let renderer = Arc::new(renderer);
        let return_value = Arc::clone(&renderer);

        self.set.lock().insert(TypeId::of::<T>(), renderer);

        return_value
    }

    pub fn get<T: RendererSetup>(&self) -> Option<Arc<T>> {
        if let Some(renderer) = self.set.lock().get(&TypeId::of::<T>()) {
            let renderer = Arc::clone(renderer);
            let arc_any = renderer as Arc<dyn Any + Send + Sync>;

            arc_any.downcast().ok()
        } else {
            None
        }
    }
}

// MARK: Renderer

pub trait RendererSetup: Any + Send + Sync {
    fn setup(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat); // todo: add some error handling
}
