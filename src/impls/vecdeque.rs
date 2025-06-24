use std::collections::VecDeque;
use std::iter;

use crate::{
    HeapSize, Tracked,
    macros::{impl_clear, impl_from, impl_new, impl_shallow_heap_size},
    tracked_value::TrackedValue,
};

impl<T> Tracked<VecDeque<T>>
where
    T: HeapSize,
{
    pub fn push_back(&mut self, value: T) {
        self.indirect_heap_memory += T::heap_size(&value);
        self.inner.push_back(value);
    }

    pub fn pop_back(&mut self) -> Option<T> {
        self.inner
            .pop_back()
            .inspect(|v| self.indirect_heap_memory -= T::heap_size(v))
    }

    pub fn push_front(&mut self, value: T) {
        self.indirect_heap_memory += T::heap_size(&value);
        self.inner.push_front(value);
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.inner
            .pop_front()
            .inspect(|v| self.indirect_heap_memory -= T::heap_size(v))
    }

    pub fn insert(&mut self, index: usize, value: T) {
        self.indirect_heap_memory += T::heap_size(&value);
        self.inner.insert(index, value);
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        self.inner
            .remove(index)
            .inspect(|value| self.indirect_heap_memory -= T::heap_size(value))
    }

    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.inner.retain(|v| {
            if f(v) {
                true
            } else {
                self.indirect_heap_memory -= T::heap_size(v);
                false
            }
        });
    }

    pub fn resize_with<F>(&mut self, new_len: usize, mut f: F)
    where
        F: FnMut() -> T,
    {
        let len = self.inner.len();
        if new_len > len {
            self.inner.extend(
                iter::repeat_with(|| {
                    let val = f();
                    self.indirect_heap_memory += T::heap_size(&val);
                    val
                })
                .take(new_len - len),
            );
        } else {
            self.truncate(new_len);
        }
    }

    pub fn truncate(&mut self, new_len: usize) {
        if new_len >= self.inner.len() {
            return;
        }
        for val in &self.inner.split_off(new_len) {
            self.indirect_heap_memory -= T::heap_size(val);
        }
    }

    /// Consider using [`append_tracked(...)`].
    pub fn append(&mut self, other: &mut VecDeque<T>) {
        for elem in &*other {
            self.indirect_heap_memory += elem.heap_size();
        }
        self.inner.append(other);
    }

    pub fn append_tracked(&mut self, other: &mut Self) {
        self.indirect_heap_memory += other.indirect_heap_memory;
        self.inner.append(&mut other.inner);
    }

    pub fn swap_remove_back(&mut self, index: usize) -> Option<T> {
        self.inner
            .swap_remove_back(index)
            .inspect(|value| self.indirect_heap_memory -= T::heap_size(value))
    }

    pub fn swap_remove_front(&mut self, index: usize) -> Option<T> {
        self.inner
            .swap_remove_front(index)
            .inspect(|value| self.indirect_heap_memory -= T::heap_size(value))
    }

    pub fn get_mut(&mut self, index: usize) -> Option<TrackedValue<'_, T>> {
        self.inner
            .get_mut(index)
            .map(|v| TrackedValue::new(&mut self.indirect_heap_memory, v))
    }
}

impl<T> Tracked<VecDeque<T>>
where
    T: HeapSize + Clone,
{
    pub fn resize(&mut self, new_len: usize, value: T) {
        let len = self.inner.len();
        if new_len > len {
            let n = new_len - len;
            self.indirect_heap_memory += value.heap_size() * n;
            self.inner.extend(iter::repeat_n(value, n));
        } else {
            self.truncate(new_len);
        }
    }
}

impl_new!(VecDeque<T>);
impl_clear!(VecDeque<T>);
impl_from!(VecDeque<T>, |v| T::heap_size(v));
impl_shallow_heap_size!(VecDeque<T>, |v: &Self| v.capacity() * (size_of::<T>()));
