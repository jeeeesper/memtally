use crate::HeapSize;

pub struct TrackedValue<'a, V>
where
    V: HeapSize,
{
    mem_tracker: &'a mut usize,
    value: &'a mut V,
    size_before: usize,
}

impl<'a, V> TrackedValue<'a, V>
where
    V: HeapSize,
{
    pub(crate) fn new(tracker: &'a mut usize, value: &'a mut V) -> Self {
        let size_before = V::heap_size(&*value);
        Self {
            mem_tracker: tracker,
            value,
            size_before,
        }
    }
}

impl<V> Drop for TrackedValue<'_, V>
where
    V: HeapSize,
{
    fn drop(&mut self) {
        let size_after = V::heap_size(self.value);

        *self.mem_tracker -= self.size_before;
        *self.mem_tracker += size_after;
    }
}

impl<V> std::ops::Deref for TrackedValue<'_, V>
where
    V: HeapSize,
{
    type Target = V;
    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<V> std::ops::DerefMut for TrackedValue<'_, V>
where
    V: HeapSize,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}
