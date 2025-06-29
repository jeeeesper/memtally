use std::{
    collections::{BinaryHeap, binary_heap::PeekMut},
    ops::Deref,
};

use crate::{
    HeapSize, Tracked,
    macros::{impl_clear, impl_from, impl_new, impl_shallow_heap_size},
    tracked_value::TrackedValue,
};

impl<T> Tracked<BinaryHeap<T>>
where
    T: Ord + HeapSize,
{
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

impl_new!(BinaryHeap<T>, T: Ord);
impl_clear!(BinaryHeap<T>);
impl_from!(BinaryHeap<T>, |v| T::heap_size(v));
impl_shallow_heap_size!(BinaryHeap<T>, |v: &Self| v.capacity() * size_of::<T>());

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
