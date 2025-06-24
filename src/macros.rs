// == Macros that make implementation easier ==

macro_rules! impl_new {
    ($name:ident<$($gen:ident),*>) => {
        impl<$($gen),*> Tracked<$name<$($gen),*>>
        {
            #[must_use]
            pub fn new() -> Self {
                Self {
                    inner: $name::default(),
                    indirect_heap_memory: 0,
                }
            }
        }
    };
    ($name:ident<$($gen:ident),*>, $($where_clause:tt)*) => {
        impl<$($gen),*> Tracked<$name<$($gen),*>>
        where $($where_clause)*
        {
            #[must_use]
            pub fn new() -> Self {
                Self {
                    inner: $name::<$($gen),*>::default(),
                    indirect_heap_memory: 0,
                }
            }
        }
    };
}
pub(crate) use impl_new;

macro_rules! impl_clear {
    ($name:ident<$($gen:ident),*>) => {
        impl<$($gen),*> Tracked<$name<$($gen),*>> {
            pub fn clear(&mut self) {
                self.indirect_heap_memory = 0;
                self.inner.clear();
            }
        }
    };
}
pub(crate) use impl_clear;

macro_rules! impl_from {
    ($name:ident<$($gen:ident),*>, $fn:expr) => {
        impl_from!($name<$($gen),*>, $fn, $($gen),*);
    };
    ($name:ident<$($gen:ident),*>, $fn:expr, $($bounds:ident),*) => {
        impl<$($gen),*> From<$name<$($gen),*>> for Tracked<$name<$($gen),*>>
        where $($bounds: HeapSize),*
        {
            fn from(value: $name<$($gen),*>) -> Self {
                let indirect_heap_memory = value.iter().map($fn).sum();
                Self {
                    inner: value,
                    indirect_heap_memory,
                }
            }
        }
    };
}
pub(crate) use impl_from;

macro_rules! impl_shallow_heap_size {
    ($name:ident<$($gen:ident),*>, $size:expr) => {
        impl<$($gen),*> crate::ShallowHeapSize for $name<$($gen),*> {
            fn shallow_heap_size(&self) -> usize {
                use std::mem::size_of;
                $size(self)
            }
        }
    };
}
pub(crate) use impl_shallow_heap_size;
