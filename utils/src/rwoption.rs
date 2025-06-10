use std::ops::{Deref, DerefMut};

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub struct RwOption<T> {
    inner: RwLock<Option<T>>,
}

/// Only use this after ensuring that the inner value is not `None`.
pub struct RwOptionReadGuard<'a, T> {
    inner: RwLockReadGuard<'a, Option<T>>,
}

impl<T> Deref for RwOptionReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.inner.as_ref().unwrap_unchecked() }
    }
}

/// Only use this after ensuring that the inner value is not `None`.
pub struct RwOptionWriteGuard<'a, T> {
    inner: RwLockWriteGuard<'a, Option<T>>,
}

impl<T> Deref for RwOptionWriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.inner.as_ref().unwrap_unchecked() }
    }
}

impl<T> DerefMut for RwOptionWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.as_mut().unwrap()
    }
}

impl<T> Default for RwOption<T> {
    fn default() -> Self {
        RwOption::new()
    }
}

impl<T> RwOption<T> {
    pub fn new() -> Self {
        RwOption {
            inner: RwLock::new(None),
        }
    }

    pub fn set(&self, value: T) {
        let mut lock = self.inner.write();
        *lock = Some(value);
    }

    pub fn get(&self) -> Option<RwOptionReadGuard<T>> {
        let read_lock = self.inner.read();
        if read_lock.is_some() {
            Some(RwOptionReadGuard { inner: read_lock })
        } else {
            None
        }
    }

    pub fn get_or_insert(&self, value: T) -> RwOptionReadGuard<T> {
        self.get_or_insert_with(|| value)
    }

    pub fn get_or_insert_default(&self) -> RwOptionReadGuard<T>
    where
        T: Default,
    {
        self.get_or_insert_with(T::default)
    }

    pub fn get_or_insert_with<F>(&self, f: F) -> RwOptionReadGuard<T>
    where
        F: FnOnce() -> T,
    {
        // check if the value is already set
        {
            let read_lock = self.inner.read();
            if read_lock.is_some() {
                return RwOptionReadGuard { inner: read_lock };
            }
        }

        // get the write lock to set the value
        let mut write_lock = self.inner.write();
        // check again after acquiring the write lock
        if write_lock.is_none() {
            // if it was still None, set the value using the provided function
            *write_lock = Some(f());
        }

        // return the read lock
        RwOptionReadGuard {
            inner: RwLockWriteGuard::downgrade(write_lock),
        }
    }

    pub fn is_some(&self) -> bool {
        self.inner.read().is_some()
    }

    pub fn is_none(&self) -> bool {
        self.inner.read().is_none()
    }

    pub fn is_some_and(&self, f: impl FnOnce(&T) -> bool) -> bool {
        self.inner.read().as_ref().is_some_and(f)
    }

    pub fn take(&self) -> Option<T> {
        let mut lock = self.inner.write();
        lock.take()
    }
}
