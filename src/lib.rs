mod impls;
mod macros;
mod tracked_value;

#[derive(Default, Debug, Eq)]
pub struct Tracked<T> {
    inner: T,
    indirect_heap_memory: usize,
}

impl<T> Tracked<T> {
    /// Get the underlying collection. This discards the memory counter.
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Get a reference to the underlying collection.
    pub fn inner(&self) -> &T {
        &self.inner
    }
}

impl<C> std::ops::Deref for Tracked<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<C> AsRef<C> for Tracked<C> {
    fn as_ref(&self) -> &C {
        &self.inner
    }
}

impl<C: PartialEq> PartialEq for Tracked<C> {
    fn eq(&self, other: &Self) -> bool {
        // Do not compare heap usage
        self.inner.eq(&other.inner)
    }
}

impl<C: PartialOrd> PartialOrd for Tracked<C> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.inner.partial_cmp(&other.inner)
    }
}

impl<C: Ord> Ord for Tracked<C> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

// Creation

impl<C, T: HeapSize> FromIterator<T> for Tracked<C>
where
    C: FromIterator<T>,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut indirect_heap_memory = 0;
        let inner: C = iter
            .into_iter()
            .inspect(|v| indirect_heap_memory += T::heap_size(v))
            .collect();
        Self {
            inner,
            indirect_heap_memory,
        }
    }
}

impl<C: Extend<T>, T: HeapSize> Extend<T> for Tracked<C> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.inner.extend(
            iter.into_iter()
                .inspect(|v| self.indirect_heap_memory += T::heap_size(v)),
        );
    }
}

/// Used for containers to report what they allocate themselves, but not
/// their elements. For example, a `Vec<T>` with capacity 16 allocates `16 *
/// std::mem::size_of::<T>()` directly. But whatever the elements of type `T`
/// might allocate is not considered here.
trait ShallowHeapSize {
    #[must_use]
    fn shallow_heap_size(&self) -> usize;
}

/// Used to query heap size of collection elements.
pub trait HeapSize {
    #[must_use]
    fn heap_size(&self) -> usize;
}

pub trait MemSize {
    #[must_use]
    fn mem_size(&self) -> usize;
}

impl<T: Sized + HeapSize> MemSize for T {
    fn mem_size(&self) -> usize {
        std::mem::size_of::<Self>() + (*self).heap_size()
    }
}

// == Compatibility layers (multiple are possible)
#[cfg(feature = "get-size")]
pub use get_size;
#[cfg(feature = "get-size")]
impl<C: ShallowHeapSize> get_size::GetSize for Tracked<C> {
    fn get_heap_size(&self) -> usize {
        self.inner.shallow_heap_size() + self.indirect_heap_memory
    }
}

#[cfg(feature = "get-size2")]
pub use get_size2;
#[cfg(feature = "get-size2")]
impl<C: ShallowHeapSize> get_size2::GetSize for Tracked<C> {
    fn get_heap_size(&self) -> usize {
        self.inner.shallow_heap_size() + self.indirect_heap_memory
    }
}

#[cfg(feature = "memuse")]
pub use memuse;
#[cfg(feature = "memuse")]
impl<C: ShallowHeapSize> memuse::DynamicUsage for Tracked<C> {
    fn dynamic_usage(&self) -> usize {
        self.inner.shallow_heap_size() + self.indirect_heap_memory
    }

    fn dynamic_usage_bounds(&self) -> (usize, Option<usize>) {
        // TODO Is usage even a lower bound? And how could we get some upper bound?
        let usage = (*self).dynamic_usage();
        (usage, None)
    }
}

// == Compatibility for reading heap usage from elements -- only one is chosen.

cfg_if::cfg_if! {
if #[cfg(feature = "get-size")] {
    impl<T: get_size::GetSize> HeapSize for T {
        fn heap_size(&self) -> usize {
            T::get_heap_size(self)
        }
    }
} else if #[cfg(feature = "get-size2")] {
    impl<T: get_size2::GetSize> HeapSize for T {
        fn heap_size(&self) -> usize {
            T::get_heap_size(self)
        }
    }
} else if #[cfg(feature = "memuse")] {
    impl<T: memuse::DynamicUsage> HeapSize for T {
        fn heap_size(&self) -> usize {
            T::dynamic_usage(self)
        }
    }
} else {
    // This is the case that we still want to cover, so that we actually have a
    // public api to all the bookkeeping we're doing.
    impl<C: ShallowHeapSize> HeapSize for Tracked<C> {
        fn heap_size(&self) -> usize {
            self.inner.shallow_heap_size() + self.indirect_heap_memory
        }
    }
}}
