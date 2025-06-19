fn main() {
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
}
