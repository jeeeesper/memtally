use std::{
    borrow::Borrow,
    collections::{BTreeMap, btree_map::Entry},
};

use crate::{HeapSize, ShallowHeapSize, Tracked, tracked_value::TrackedValue};

impl<K, V> Tracked<BTreeMap<K, V>>
where
    K: Ord + HeapSize,
    V: HeapSize,
{
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        match self.inner.entry(key) {
            Entry::Occupied(mut o) => {
                let old_v = o.get_mut();
                let old_size = V::heap_size(old_v);
                let new_size = V::heap_size(&value);

                self.indirect_heap_memory -= old_size;
                self.indirect_heap_memory += new_size;

                // key is not updated

                Some(std::mem::replace(old_v, value))
            }
            Entry::Vacant(v) => {
                let k = v.key();
                let k_size = K::heap_size(k);
                let v_size = V::heap_size(&value);

                self.indirect_heap_memory += k_size + v_size;
                v.insert(value);
                None
            }
        }
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.inner
            .remove_entry(key)
            .inspect(|(k, v)| {
                self.indirect_heap_memory -= K::heap_size(k);
                self.indirect_heap_memory -= V::heap_size(v);
            })
            .map(|(_, v)| v)
    }

    pub fn remove_entry<Q>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.inner.remove_entry(key).inspect(|(k, v)| {
            self.indirect_heap_memory -= K::heap_size(k);
            self.indirect_heap_memory -= V::heap_size(v);
        })
    }

    pub fn entry(&mut self, key: K) -> TrackedEntry<'_, K, V> {
        match self.inner.entry(key) {
            std::collections::btree_map::Entry::Occupied(o) => {
                TrackedEntry::Occupied(TrackedOccupiedEntry {
                    tracker: &mut self.indirect_heap_memory,
                    entry: o,
                })
            }
            std::collections::btree_map::Entry::Vacant(v) => {
                TrackedEntry::Vacant(TrackedVacantEntry {
                    tracker: &mut self.indirect_heap_memory,
                    entry: v,
                })
            }
        }
    }

    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<TrackedValue<'_, V>>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.inner
            .get_mut(key)
            .map(|v| TrackedValue::new(&mut self.indirect_heap_memory, v))
    }
}

impl<K, V> From<BTreeMap<K, V>> for Tracked<BTreeMap<K, V>>
where
    K: HeapSize,
    V: HeapSize,
{
    fn from(value: BTreeMap<K, V>) -> Self {
        let indirect_heap_memory = value
            .iter()
            .map(|(k, v)| K::heap_size(k) + V::heap_size(v))
            .sum();
        Self {
            inner: value,
            indirect_heap_memory,
        }
    }
}

pub enum TrackedEntry<'a, K, V> {
    Occupied(TrackedOccupiedEntry<'a, K, V>),
    Vacant(TrackedVacantEntry<'a, K, V>),
}

pub struct TrackedOccupiedEntry<'a, K, V> {
    tracker: &'a mut usize,
    entry: std::collections::btree_map::OccupiedEntry<'a, K, V>,
}

impl<'a, K, V> TrackedOccupiedEntry<'a, K, V>
where
    K: Ord + HeapSize,
    V: HeapSize,
{
    #[must_use]
    pub fn get(&self) -> &V {
        self.entry.get()
    }

    #[must_use]
    pub fn into_mut(self) -> TrackedValue<'a, V> {
        TrackedValue::new(self.tracker, self.entry.into_mut())
    }

    pub fn insert(&mut self, value: V) -> V {
        let old_value = self.entry.insert(value);
        let old_size = V::heap_size(&old_value);
        let new_size = V::heap_size(self.entry.get());

        *self.tracker -= old_size;
        *self.tracker += new_size;

        old_value
    }

    #[allow(clippy::must_use_candidate)]
    pub fn remove(self) -> V {
        let key_size = K::heap_size(self.entry.key());
        let val_size = V::heap_size(self.entry.get());
        *self.tracker -= key_size + val_size;
        self.entry.remove()
    }
}

pub struct TrackedVacantEntry<'a, K, V> {
    tracker: &'a mut usize,
    entry: std::collections::btree_map::VacantEntry<'a, K, V>,
}

impl<'a, K, V> TrackedVacantEntry<'a, K, V>
where
    K: Ord + HeapSize,
    V: HeapSize,
{
    pub fn insert(self, value: V) -> &'a mut V {
        let k_size = K::heap_size(self.entry.key());
        let v_size = V::heap_size(&value);
        *self.tracker += k_size + v_size;
        self.entry.insert(value)
    }
}

impl<K, V> ShallowHeapSize for BTreeMap<K, V> {
    fn shallow_heap_size(&self) -> usize {
        use std::mem::size_of;

        // An estimation.
        self.len() * (size_of::<K>() + size_of::<V>() + size_of::<usize>())
    }
}
