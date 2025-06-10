use std::{
    hash::{BuildHasher, Hash},
    num::NonZero,
    sync::Arc,
};

use lru::{DefaultHasher, LruCache};
use parking_lot::Mutex;

pub struct ConcLru<K, V, S = DefaultHasher> {
    inner: Mutex<LruCache<K, Arc<V>, S>>,
}

impl<K: Hash + Eq, V> ConcLru<K, V> {
    pub fn new(cap: NonZero<usize>) -> Self {
        Self {
            inner: Mutex::new(LruCache::new(cap)),
        }
    }

    pub fn unbounded() -> Self {
        Self {
            inner: Mutex::new(LruCache::unbounded()),
        }
    }
}

impl<K: Hash + Eq, V, S: BuildHasher> ConcLru<K, V, S> {
    pub fn with_hasher(cap: NonZero<usize>, hasher: S) -> Self {
        Self {
            inner: Mutex::new(LruCache::with_hasher(cap, hasher)),
        }
    }

    pub fn unbounded_with_hasher(hasher: S) -> Self {
        Self {
            inner: Mutex::new(LruCache::unbounded_with_hasher(hasher)),
        }
    }
}

impl<K: Hash + Eq, V, S: BuildHasher> ConcLru<K, V, S> {
    pub fn put(&self, key: K, value: V) {
        let mut cache = self.inner.lock();
        cache.put(key, Arc::new(value));
    }

    pub fn get(&self, key: &K) -> Option<Arc<V>> {
        let mut cache = self.inner.lock();
        cache.get(key).cloned()
    }

    pub fn get_or_insert(&self, key: K, value: V) -> Arc<V> {
        self.get_or_insert_with(key, || value)
    }

    pub fn get_or_insert_default(&self, key: K) -> Arc<V>
    where
        V: Default,
    {
        self.get_or_insert_with(key, V::default)
    }

    pub fn get_or_insert_with<F>(&self, key: K, f: F) -> Arc<V>
    where
        F: FnOnce() -> V,
    {
        let mut cache = self.inner.lock();
        if let Some(v) = cache.get(&key) {
            v.clone()
        } else {
            let arc_value = Arc::new(f());
            cache.put(key, arc_value.clone());
            arc_value
        }
    }

    pub fn get_or_insert_arc(&self, key: K, value: Arc<V>) -> Arc<V> {
        let mut cache = self.inner.lock();
        if let Some(v) = cache.get(&key) {
            v.clone()
        } else {
            cache.put(key, value.clone());
            value
        }
    }
}
