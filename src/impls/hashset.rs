use std::{
    borrow::Borrow,
    collections::HashSet,
    hash::{BuildHasher, Hash, RandomState},
};

use crate::{HeapSize, Tracked, ShallowHeapSize};

impl<T> Tracked<HashSet<T, RandomState>> {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

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

impl<T, S: BuildHasher> From<HashSet<T, S>> for Tracked<HashSet<T, S>>
where
    T: HeapSize,
{
    fn from(value: HashSet<T, S>) -> Self {
        let indirect_heap_memory = value.iter().map(|k| T::heap_size(k)).sum();
        Self {
            inner: value,
            indirect_heap_memory,
        }
    }
}

impl<T, S> ShallowHeapSize for HashSet<T, S> {
    fn shallow_heap_size(&self) -> usize {
        use std::mem::size_of;

        // An estimation.
        self.capacity() * (size_of::<T>() + size_of::<usize>())
    }
}
