# MemTally

**MemTally** provides `Tracked<C>`, a lightweight wrapper around standard Rust collection types (e.g., `HashMap`, `Vec`, `HashSet`) that tracks the heap-allocated memory used by their elements.
This enables constant-time estimation of memory usage, without the need to again iterate through all elements.

---

## Features

- Track additional heap usage of the elements of a collection
- Works with (some) standard Rust collections
- Constant-time memory usage queries
- Supports integration with third-party crates like [`get-size`](https://crates.io/crates/get-size), [`get-size2`](https://crates.io/crates/get-size2), and [`memuse`](https://crates.io/crates/memuse)

---

## Usage

To use MemTally, wrap your collection with `Tracked<C>`. Elements must implement the `HeapSize` trait.

```rust
use memtally::{HeapSize, Tracked};
use std::collections::HashSet;

// Example element type implementing HeapSize
#[derive(Hash, Eq, PartialEq)]
struct Item {
    data: String,
}

impl HeapSize for Item {
    fn heap_size(&self) -> usize {
        self.data.capacity()
    }
}

let mut set = Tracked::from(HashSet::new());
set.insert(Item {
    data: "hello".into(),
});

println!("Total heap usage: {} bytes", set.heap_size());
```

All immutable methods from the underlying collection are accessible via Deref. Mutating operations must be performed through Tracked.

## Feature Flags

To avoid writing manual HeapSize impls for common types, enable one of the following features to use automatic implementations from third-party crates:
- `get-size`: Uses the `get-size` crate
- `get-size2`: Uses the `get-size2` crate
- `memuse`: Uses the `memuse` crate

Enable them in your Cargo.toml:

```
memtally = { version = "0.1.0", features = ["get-size"] }
```

## Caveats
This crate is currently an early prototype. APIs may change, and the accuracy of memory estimation has not been properly tested, and neither have all the mutating methods been verified for correctness.
Contributions and feedback are welcome.
