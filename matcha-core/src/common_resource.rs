use std::{
    any::{Any, TypeId},
    sync::Arc,
};

// note: consider using `dashmap` or `scc` if performance is an issue

use dashmap::DashMap;

#[derive(Default)]
pub struct CommonResource {
    // resource: Mutex<FxHashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
    resource: DashMap<TypeId, Arc<dyn Any + Send + Sync>, fxhash::FxBuildHasher>,
}

impl CommonResource {
    pub fn new() -> Self {
        Default::default()
    }

    // insert() and get() methods are not recommended for public use

    // pub fn insert<T>(&self, renderer: T)
    // where
    //     T: Send + Sync + 'static,
    // {
    //     self.resource
    //         .insert(TypeId::of::<T>(), Arc::new(renderer));
    // }

    // pub fn get<T>(&self) -> Option<Arc<T>>
    // where
    //     T: Send + Sync + 'static,
    // {
    //     self.resource
    //         .get(&TypeId::of::<T>())
    //         .and_then(|v| v.downcast_ref::<Arc<T>>().cloned())
    // }

    pub fn get_or_insert<T>(&self, v: T) -> Arc<T>
    where
        T: Send + Sync + 'static,
    {
        self.get_or_insert_with(|| v)
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
        self.resource
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Arc::new(f()))
            .clone()
            .downcast()
            .expect("Type map in `CommonResource` should ensure `key == value.type_id()`")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TypeA;
    struct TypeB;
    #[derive(Default)]
    struct TypeC {
        v: u32,
    }

    #[test]
    fn test_common_resource() {
        let resource = CommonResource::new();

        let a = resource.get_or_insert(TypeA);
        let b = resource.get_or_insert_with(|| TypeB);
        let c = resource.get_or_insert_default::<TypeC>();

        assert!(TypeId::of::<Arc<TypeA>>() == a.type_id());
        assert!(TypeId::of::<Arc<TypeB>>() == b.type_id());
        assert!(TypeId::of::<Arc<TypeC>>() == c.type_id());

        let c2 = resource.get_or_insert(TypeC { v: 42 });
        assert_eq!(c2.v, u32::default());
    }
}
