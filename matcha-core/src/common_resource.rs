use std::{
    any::{Any, TypeId},
    sync::Arc,
};

// note: consider using `dashmap` if performance is an issue

use fxhash::FxHashMap;
use parking_lot::Mutex;

#[derive(Default)]
pub struct CommonResource {
    map: Mutex<FxHashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
}

impl CommonResource {
    pub fn new() -> Self {
        Self {
            map: Mutex::new(FxHashMap::default()),
        }
    }

    // pub fn insert<T>(&self, renderer: Arc<T>)
    // where
    //     T: Send + Sync + 'static,
    // {
    //     let mut map = self.map.lock();
    //     map.insert(TypeId::of::<T>(), renderer);
    // }

    pub fn get_or_insert<T>(&self, value: T) -> Arc<T>
    where
        T: Send + Sync + 'static,
    {
        self.get_or_insert_with(|| value)
    }

    pub fn get_or_insert_default<T>(&self) -> Arc<T>
    where
        T: Default + Send + Sync + 'static,
    {
        self.get_or_insert_with(T::default)
    }

    pub fn get_or_insert_with<T, F>(&self, f: F) -> Arc<T>
    where
        T: Send + Sync + 'static,
        F: FnOnce() -> T,
    {
        let mut map = self.map.lock();

        if let Some(renderer) = map.get(&TypeId::of::<T>()) {
            let renderer = Arc::clone(renderer);
            let arc_any = renderer as Arc<dyn Any + Send + Sync>;
            return arc_any.downcast().unwrap();
        }

        // If not, create a new
        let renderer = Arc::new(f());
        let return_value = Arc::clone(&renderer);

        map.insert(TypeId::of::<T>(), renderer);

        return_value
    }

    // pub fn get<T>(&self) -> Option<Arc<T>>
    // where
    //     T: Send + Sync + 'static,
    // {
    //     let map = self.map.lock();
    //     if let Some(renderer) = map.get(&TypeId::of::<T>()) {
    //         let renderer = Arc::clone(renderer);
    //         let arc_any = renderer as Arc<dyn Any + Send + Sync>;

    //         arc_any.downcast().ok()
    //     } else {
    //         None
    //     }
    // }
}
