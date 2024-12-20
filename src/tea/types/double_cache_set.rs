use std::collections::HashMap;

pub struct DoubleSetCache<K, V>
where
    K: Eq + std::hash::Hash + Clone,
{
    a: HashMap<K, Box<V>>,
    b: HashMap<K, Box<V>>,
    /// true if a is valid, false if b is valid.
    current_valid_cache: bool,
    /// current frame count.
    current_frame: u64,
}

impl<K, V> DoubleSetCache<K, V>
where
    K: Eq + std::hash::Hash + Clone,
{
    const A_VALID: bool = true;
    const B_VALID: bool = false;

    pub fn new() -> Self {
        Self {
            a: HashMap::new(),
            b: HashMap::new(),
            current_valid_cache: true,
            current_frame: 0,
        }
    }

    pub fn clear(&mut self) {
        self.a.clear();
        self.b.clear();
    }

    pub fn get_or_insert(&mut self, key: K, frame: u64, value: V) -> &mut V {
        self.get_or_insert_with(key, frame, || value)
    }

    pub fn get_or_insert_with<F>(&mut self, k: K, frame: u64, f: F) -> &mut V
    where
        F: FnOnce() -> V,
    {
        // swap cache if frame updated.
        // if a was valid when frame updated, clear b and switch valid cache to b.
        if self.current_frame != frame {
            self.current_frame = frame;
            // clear cache and switch.
            match self.current_valid_cache {
                Self::A_VALID => {
                    self.b.clear();
                    self.current_valid_cache = Self::B_VALID;
                }
                Self::B_VALID => {
                    self.a.clear();
                    self.current_valid_cache = Self::A_VALID;
                }
            }
        }

        // get cache.
        let (cache, back_cache) = match self.current_valid_cache {
            Self::A_VALID => (&mut self.a, &mut self.b),
            Self::B_VALID => (&mut self.b, &mut self.a),
        };

        // get or insert value.
        cache
            .entry(k.clone())
            .or_insert_with(|| back_cache.remove(&k).unwrap_or_else(|| Box::new(f())))
    }

    #[cfg(test)]
    fn get_from_valid(&self, k: &K) -> Option<&V> {
        match self.current_valid_cache {
            Self::A_VALID => self.a.get(k).map(|v| &**v),
            Self::B_VALID => self.b.get(k).map(|v| &**v),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut cache = DoubleSetCache::new();

        cache.get_or_insert_with(0, 0, || "a");
        cache.get_or_insert_with(1, 0, || "b");
        cache.get_or_insert_with(2, 0, || "c");
        assert_eq!(cache.get_from_valid(&0), Some(&"a"));
        assert_eq!(cache.get_from_valid(&1), Some(&"b"));
        assert_eq!(cache.get_from_valid(&2), Some(&"c"));
        assert_eq!(cache.get_from_valid(&3), None);

        cache.get_or_insert_with(0, 1, || "d"); // will not be inserted.
        assert_eq!(cache.get_from_valid(&0), Some(&"a"));
        assert_eq!(cache.get_from_valid(&1), None);
        assert_eq!(cache.get_from_valid(&2), None);
        cache.get_or_insert_with(1, 1, || "e"); // will not be inserted.
        assert_eq!(cache.get_from_valid(&0), Some(&"a"));
        assert_eq!(cache.get_from_valid(&1), Some(&"b"));
        assert_eq!(cache.get_from_valid(&2), None);
        cache.get_or_insert_with(2, 1, || "f"); // will not be inserted.
        assert_eq!(cache.get_from_valid(&0), Some(&"a"));
        assert_eq!(cache.get_from_valid(&1), Some(&"b"));
        assert_eq!(cache.get_from_valid(&2), Some(&"c"));

        cache.clear();

        assert_eq!(cache.get_from_valid(&0), None);
        assert_eq!(cache.get_from_valid(&1), None);
        assert_eq!(cache.get_from_valid(&2), None);
    }
}
