use std::{
    borrow::Borrow,
    collections::{HashMap, hash_map::Entry},
    hash::{BuildHasher, Hash},
};

use crate::{
    HeapSize, Tracked,
    macros::{impl_clear, impl_from, impl_new, impl_shallow_heap_size},
    tracked_value::TrackedValue,
};

impl<K, V, S> Tracked<HashMap<K, V, S>>
where
    K: Eq + Hash + HeapSize,
    V: HeapSize,
    S: BuildHasher,
{
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.indirect_heap_memory += V::heap_size(&value);
        match self.inner.entry(key) {
            Entry::Occupied(mut o) => {
                // Subtract old value
                self.indirect_heap_memory -= V::heap_size(o.get());
                let old = o.insert(value);
                Some(old)
            }
            Entry::Vacant(v) => {
                // Add key
                self.indirect_heap_memory += K::heap_size(v.key());
                v.insert(value);
                None
            }
        }
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
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
        Q: Hash + Eq + ?Sized,
    {
        self.inner.remove_entry(key).inspect(|(k, v)| {
            self.indirect_heap_memory -= K::heap_size(k);
            self.indirect_heap_memory -= V::heap_size(v);
        })
    }

    pub fn entry(&mut self, key: K) -> TrackedEntry<'_, K, V> {
        match self.inner.entry(key) {
            std::collections::hash_map::Entry::Occupied(o) => {
                TrackedEntry::Occupied(TrackedOccupiedEntry {
                    tracker: &mut self.indirect_heap_memory,
                    entry: o,
                })
            }
            std::collections::hash_map::Entry::Vacant(v) => {
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
        Q: Hash + Eq + ?Sized,
    {
        self.inner
            .get_mut(key)
            .map(|v| TrackedValue::new(&mut self.indirect_heap_memory, v))
    }
}

impl_new!(HashMap<K, V, S>, S: BuildHasher + Default);
impl_clear!(HashMap<K, V, S>);
impl_from!(HashMap<K, V, S>, |(k, v)| K::heap_size(k) + V::heap_size(v), K, V);
impl_shallow_heap_size!(HashMap<K, V, S>, |v: &Self| v.capacity() * (size_of::<K>() + size_of::<V>() + size_of::<usize>()));

pub enum TrackedEntry<'a, K, V> {
    Occupied(TrackedOccupiedEntry<'a, K, V>),
    Vacant(TrackedVacantEntry<'a, K, V>),
}

pub struct TrackedOccupiedEntry<'a, K, V> {
    tracker: &'a mut usize,
    entry: std::collections::hash_map::OccupiedEntry<'a, K, V>,
}

impl<'a, K, V> TrackedOccupiedEntry<'a, K, V>
where
    K: Eq + Hash + HeapSize,
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

    #[allow(
        clippy::must_use_candidate,
        reason = "Mostly executed for side effects"
    )]
    pub fn remove(self) -> V {
        let key_size = K::heap_size(self.entry.key());
        let val_size = V::heap_size(self.entry.get());
        *self.tracker -= key_size + val_size;
        self.entry.remove()
    }
}

pub struct TrackedVacantEntry<'a, K, V> {
    tracker: &'a mut usize,
    entry: std::collections::hash_map::VacantEntry<'a, K, V>,
}

impl<'a, K, V> TrackedVacantEntry<'a, K, V>
where
    K: HeapSize,
    V: HeapSize,
{
    pub fn insert(self, value: V) -> &'a mut V {
        let k_size = K::heap_size(self.entry.key());
        let v_size = V::heap_size(&value);
        *self.tracker += k_size + v_size;
        self.entry.insert(value)
    }
}
