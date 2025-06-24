use std::{
    borrow::Borrow,
    collections::HashSet,
    hash::{BuildHasher, Hash},
};

use crate::{
    HeapSize, Tracked,
    macros::{impl_clear, impl_from, impl_new, impl_shallow_heap_size},
};

impl<T, S> Tracked<HashSet<T, S>>
where
    T: Eq + Hash + HeapSize,
    S: BuildHasher,
{
    pub fn insert(&mut self, key: T) -> bool {
        let key_size = T::heap_size(&key);
        if self.inner.insert(key) {
            self.indirect_heap_memory += key_size;
            true
        } else {
            false
        }
    }

    pub fn remove<Q>(&mut self, key: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if let Some(k) = self.inner.take(key) {
            self.indirect_heap_memory -= T::heap_size(&k);
            true
        } else {
            false
        }
    }

    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.inner.retain(|key| {
            if f(key) {
                true
            } else {
                self.indirect_heap_memory -= T::heap_size(key);
                false
            }
        });
    }
}

impl_new!(HashSet<T, S>, S: BuildHasher + Default);
impl_clear!(HashSet<T, S>);
impl_from!(HashSet<T, S>, |v| T::heap_size(v), T);
impl_shallow_heap_size!(HashSet<T, S>, |v: &Self| v.capacity()
    * (size_of::<T>() + size_of::<usize>()));
