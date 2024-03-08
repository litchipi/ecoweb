use std::collections::HashMap;

// TODO    Create a smart cache to keep frequently used data
pub struct Cache<K, V> {
    data: HashMap<K, V>,
}

impl<K, V> Cache<K, V> {
    pub fn empty() -> Cache<K, V> {
        Cache {
            data: HashMap::new(),
        }
    }
}
