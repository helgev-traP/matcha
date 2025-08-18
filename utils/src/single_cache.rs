pub struct SingleCache<K: PartialEq, V> {
    data: Option<(K, V)>,
}

impl<K: PartialEq, V> Default for SingleCache<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: PartialEq, V> SingleCache<K, V> {
    pub fn new() -> Self {
        SingleCache { data: None }
    }

    pub fn set(&mut self, key: K, value: V) {
        self.data = Some((key, value));
    }

    pub fn get(&self) -> Option<(&K, &V)> {
        self.data.as_ref().map(|(k, v)| (k, v))
    }

    pub fn get_mut(&mut self) -> Option<(&K, &mut V)> {
        self.data.as_mut().map(|(k, v)| (k as &K, v))
    }

    pub fn get_or_insert(&mut self, key: K, value: V) -> (&K, &mut V) {
        self.get_or_insert_with(key, || value)
    }

    pub fn get_or_insert_default(&mut self, key: K) -> (&K, &mut V)
    where
        V: Default,
    {
        self.get_or_insert_with(key, V::default)
    }

    pub fn get_or_insert_with<F>(&mut self, key: K, f: F) -> (&K, &mut V)
    where
        F: FnOnce() -> V,
    {
        if !self.get().is_some_and(|(k, _)| *k == key) {
            self.set(key, f());
        }

        self.get_mut()
            .expect("infallible: cache is guaranteed to be populated")
    }
}
