use std::{
    collections::{BinaryHeap, binary_heap::PeekMut},
    ops::Deref,
};

use crate::{HeapSize, ShallowHeapSize, Tracked, tracked_value::TrackedValue};

impl<T> Tracked<BinaryHeap<T>>
where
    T: Ord + HeapSize,
{
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, item: T) {
        self.indirect_heap_memory += T::heap_size(&item);
        self.inner.push(item);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.inner
            .pop()
            .inspect(|v| self.indirect_heap_memory -= T::heap_size(v))
    }

    pub fn peek_mut(&mut self) -> Option<TrackedPeekMut<'_, T>> {
        let elem = self.inner.peek_mut()?;
        Some(TrackedPeekMut {
            tracker: &mut self.indirect_heap_memory,
            elem,
        })
    }
}

pub struct TrackedPeekMut<'a, T: 'a + Ord> {
    tracker: &'a mut usize,
    elem: PeekMut<'a, T>,
}

impl<'a, T: 'a + Ord> Deref for TrackedPeekMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.elem
    }
}

impl<'a, T: 'a + Ord + HeapSize> TrackedPeekMut<'a, T> {
    pub fn get_mut(&'a mut self) -> TrackedValue<'a, T> {
        TrackedValue::new(self.tracker, &mut *self.elem)
    }

    pub fn pop(self) -> T {
        *self.tracker -= T::heap_size(&self.elem);
        PeekMut::pop(self.elem)
    }
}

impl<T> From<BinaryHeap<T>> for Tracked<BinaryHeap<T>>
where
    T: HeapSize,
{
    fn from(value: BinaryHeap<T>) -> Self {
        let indirect_heap_memory = value.iter().map(|k| T::heap_size(k)).sum();
        Self {
            inner: value,
            indirect_heap_memory,
        }
    }
}

impl<T> ShallowHeapSize for BinaryHeap<T> {
    fn shallow_heap_size(&self) -> usize {
        use std::mem::size_of;

        // An estimation.
        self.len() * (size_of::<T>() + size_of::<usize>())
    }
}
