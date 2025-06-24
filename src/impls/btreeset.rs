use std::{borrow::Borrow, collections::BTreeSet};

use crate::{
    HeapSize, Tracked,
    macros::{impl_clear, impl_from, impl_new, impl_shallow_heap_size},
};

impl<T> Tracked<BTreeSet<T>>
where
    T: Ord + HeapSize,
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
        Q: Ord + ?Sized,
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

impl_new!(BTreeSet<T>);
impl_clear!(BTreeSet<T>);
impl_from!(BTreeSet<T>, |v| T::heap_size(v));
impl_shallow_heap_size!(BTreeSet<T>, |v: &Self| v.len()
    * (size_of::<T>() + size_of::<usize>()));
