pub struct Cache<T, D>
where
    T: PartialEq + Clone,
{
    tag: Option<T>,
    data: Option<D>,
}

impl<T, D> Default for Cache<T, D>
where
    T: PartialEq + Clone,
{
    fn default() -> Self {
        Self {
            tag: None,
            data: None,
        }
    }
}

impl<T, D> Cache<T, D>
where
    T: PartialEq + Clone,
{
    pub fn new() -> Self {
        Self {
            tag: None,
            data: None,
        }
    }

    pub fn set(&mut self, tag: T, data: D) {
        self.tag = Some(tag);
        self.data = Some(data);
    }

    pub fn clear(&mut self) {
        self.tag = None;
        self.data = None;
    }

    pub fn get_tag(&self) -> Option<&T> {
        self.tag.as_ref()
    }

    pub fn tag_eq(&self, tag: &T) -> bool {
        self.tag.as_ref() == Some(tag)
    }

    pub fn get_data(&self, tag: &T) -> Option<&D> {
        if self.tag_eq(tag) {
            self.data.as_ref()
        } else {
            None
        }
    }

    pub fn get_data_mut(&mut self, tag: &T) -> Option<&mut D> {
        if self.tag_eq(tag) {
            self.data.as_mut()
        } else {
            None
        }
    }

    pub fn get_data_f(&self) -> Option<&D> {
        self.data.as_ref()
    }

    pub fn get_data_mut_f(&mut self) -> Option<&mut D> {
        self.data.as_mut()
    }

    pub fn get_data_or_insert(&mut self, tag: &T, default: D) -> &mut D
    where
        D: Default,
    {
        if self.tag_eq(tag) {
            self.data.as_mut().unwrap()
        } else {
            self.set(tag.clone(), default);
            self.data.as_mut().unwrap()
        }
    }

    pub fn get_data_or_default(&mut self, tag: &T) -> &mut D
    where
        D: Default,
    {
        if self.tag_eq(tag) {
            self.data.as_mut().unwrap()
        } else {
            self.set(tag.clone(), D::default());
            self.data.as_mut().unwrap()
        }
    }

    pub fn get_data_or_insert_with<F>(&mut self, tag: &T, f: F) -> &mut D
    where
        F: FnOnce() -> D,
    {
        if self.tag_eq(tag) {
            self.data.as_mut().unwrap()
        } else {
            self.set(tag.clone(), f());
            self.data.as_mut().unwrap()
        }
    }
}
