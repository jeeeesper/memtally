use std::ops::Deref;

use binary_heap_plus::{BinaryHeap, PeekMut};
use compare::Compare;

use crate::{HeapSize, ShallowHeapSize, Tracked, tracked_value::TrackedValue};

impl<T, C> Tracked<BinaryHeap<T, C>>
where
    T: HeapSize,
    C: Compare<T>,
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

    pub fn peek_mut(&mut self) -> Option<TrackedPeekMut<'_, T, C>> {
        let elem = self.inner.peek_mut()?;
        Some(TrackedPeekMut {
            tracker: &mut self.indirect_heap_memory,
            elem,
        })
    }

    pub fn clear(&mut self) {
        self.indirect_heap_memory = 0;
        self.inner.clear();
    }
}

pub struct TrackedPeekMut<'a, T: 'a, C: 'a + Compare<T>> {
    tracker: &'a mut usize,
    elem: PeekMut<'a, T, C>,
}

impl<'a, T: 'a, C: 'a + Compare<T>> Deref for TrackedPeekMut<'a, T, C> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.elem
    }
}

impl<'a, T: 'a + HeapSize, C: 'a + Compare<T>> TrackedPeekMut<'a, T, C> {
    pub fn get_mut(&'a mut self) -> TrackedValue<'a, T> {
        TrackedValue::new(self.tracker, &mut *self.elem)
    }

    pub fn pop(self) -> T {
        *self.tracker -= T::heap_size(&self.elem);
        PeekMut::pop(self.elem)
    }
}

impl<T, C> From<BinaryHeap<T, C>> for Tracked<BinaryHeap<T, C>>
where
    T: HeapSize,
{
    fn from(value: BinaryHeap<T, C>) -> Self {
        let indirect_heap_memory = value.iter().map(|k| T::heap_size(k)).sum();
        Self {
            inner: value,
            indirect_heap_memory,
        }
    }
}

impl<T, C> ShallowHeapSize for BinaryHeap<T, C> {
    fn shallow_heap_size(&self) -> usize {
        use std::mem::size_of;

        // An estimation.
        self.capacity() * size_of::<T>()
    }
}
