use parking_lot::RwLock;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::atomic::{AtomicUsize, Ordering};

pub trait CacheKey: Clone + Eq + Hash {}
pub trait CacheVal: Clone + Sized {}

impl<T: Clone + Eq + Hash> CacheKey for T {}
impl<T: Clone + Sized> CacheVal for T {}

pub struct Cache<K: CacheKey, V: CacheVal> {
    size_limit: usize,
    tot_size: AtomicUsize,

    data: RwLock<HashMap<K, V>>,
    count: RwLock<HashMap<K, AtomicUsize>>,
    tot_count: AtomicUsize,
}

impl<K: CacheKey, V: CacheVal> std::fmt::Debug for Cache<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tot_size = self.tot_size.load(Ordering::Relaxed);
        let nelements = self.data.read().len();
        let naccesses = self.tot_count.load(Ordering::Relaxed);
        write!(
            f,
            "Cache {{ {tot_size}/{} bytes used, {} elements, {} accesses }}",
            self.size_limit, nelements, naccesses,
        )
    }
}

impl<K: CacheKey, V: CacheVal> Cache<K, V> {
    pub fn empty(size_limit: usize) -> Cache<K, V> {
        Cache {
            size_limit,
            tot_size: AtomicUsize::new(0),
            tot_count: AtomicUsize::new(0),
            count: RwLock::new(HashMap::new()),
            data: RwLock::new(HashMap::new()),
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let tstart = std::time::Instant::now();
        self.tot_count.fetch_add(1, Ordering::Relaxed);
        let res = if let Some(data) = self.data.read().get(key) {
            self.count
                .read()
                .get(key)
                .unwrap()
                .fetch_add(1, Ordering::Relaxed);
            Some(data.clone())
        } else {
            None
        };
        log::debug!("Got data from cache in {:?}", tstart.elapsed());
        res
    }

    pub fn add(&self, key: K, val: V) {
        let tstart = std::time::Instant::now();
        let tot_size = self.tot_size.load(Ordering::Relaxed);
        let val_size = std::mem::size_of::<V>();
        if (val_size + tot_size) > self.size_limit {
            self.make_space(val_size);
        }
        self.data.write().insert(key.clone(), val);
        let mut count = self.count.write();
        if let Some(cnt) = count.get_mut(&key) {
            cnt.fetch_add(1, Ordering::Relaxed);
        } else {
            count.insert(key, AtomicUsize::new(1));
        }
        self.tot_size.fetch_add(1, Ordering::Relaxed);
        log::debug!("Added data to cache in {:?}", tstart.elapsed());
    }

    pub fn make_space(&self, size: usize) {
        // TODO    Remove elements that are not used regularly
    }
}
