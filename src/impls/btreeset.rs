use std::{borrow::Borrow, collections::BTreeSet};

use crate::{HeapSize, Tracked, ShallowHeapSize};

impl<T> Tracked<BTreeSet<T>>
where
    T: Ord + HeapSize,
{
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

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

impl<T> From<BTreeSet<T>> for Tracked<BTreeSet<T>>
where
    T: HeapSize,
{
    fn from(value: BTreeSet<T>) -> Self {
        let indirect_heap_memory = value.iter().map(|k| T::heap_size(k)).sum();
        Self {
            inner: value,
            indirect_heap_memory,
        }
    }
}

impl<T> ShallowHeapSize for BTreeSet<T> {
    fn shallow_heap_size(&self) -> usize {
        use std::mem::size_of;

        // An estimation.
        self.len() * (size_of::<T>() + size_of::<usize>())
    }
}
