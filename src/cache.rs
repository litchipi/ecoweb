use parking_lot::RwLock;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

use serde::{Deserialize, Serialize};

use crate::config::Configuration;
use crate::post::{Post, PostMetadata};

#[derive(Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    limit_size: usize,

    post_cache_weight: f64,
    post_nav_weight: f64,
    post_page_rendered: f64,
    post_metadata: f64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        CacheConfig {
            limit_size: 10 * 1024 * 1024, // 10 Mo

            post_cache_weight: 0.8,
            post_nav_weight: 0.3,
            post_page_rendered: 0.5,
            post_metadata: 0.1,
        }
    }
}

impl CacheConfig {
    fn get_cache_size(&self) -> Vec<usize> {
        let data = vec![
            self.post_cache_weight,
            self.post_nav_weight,
            self.post_page_rendered,
            self.post_metadata,
        ];
        let sum: f64 = data.iter().sum();
        data.iter()
            .map(|v| ((v / sum) * (self.limit_size as f64)) as usize)
            .collect()
    }
}

// TODO    Use channels to count the "get" events
//    Avoid having to get the "write" permission on the RwLock cache
//    Remove the need for "&mut self" on get_copy operation
pub struct CacheMap<I, T>
where
    T: CacheElement,
{
    cache: HashMap<I, T>,
    hit_counter: HashMap<I, (usize, usize)>,
    limit: usize,
    current: usize,
    nb_queries: usize,
}

#[allow(dead_code)]
impl<I: Clone + PartialEq + Eq + Debug + Hash, T: CacheElement> CacheMap<I, T> {
    pub fn empty(limit: usize) -> CacheMap<I, T> {
        CacheMap {
            cache: HashMap::new(),
            hit_counter: HashMap::new(),
            limit,
            current: 0,
            nb_queries: 0,
        }
    }

    #[allow(unreachable_code, dead_code, unused_variables)]
    pub fn add(&mut self, ind: I, val: T) {
        #[cfg(feature = "hot_reloading")]
        return;

        let sz = val.len();
        if self.current + sz > self.limit {
            self.purge(sz);
        }
        self.hit_counter.insert(ind.clone(), (0, self.nb_queries));
        self.cache.insert(ind.clone(), val);
        self.current += sz;
        log::debug!(
            "Added {:?} to cache: {}/{} memory used ({} items)",
            ind,
            self.current,
            self.limit,
            self.cache.len()
        );
    }

    #[allow(unreachable_code, dead_code, unused_variables)]
    pub fn get_copy(&mut self, ind: &I) -> Option<T> {
        #[cfg(feature = "hot_reloading")]
        return None;

        let Some(val) = self.cache.get(ind) else {
            return None;
        };
        let cnt = self.hit_counter.get_mut(ind).unwrap();
        cnt.0 += 1;
        self.nb_queries += 1;
        log::debug!(
            "Queried {:?} from cache: {}/{} queries for this content",
            ind,
            cnt.0,
            self.nb_queries - cnt.1
        );
        Some(val.clone())
    }

    pub fn purge(&mut self, mut needed: usize) {
        for (key, (nb, off)) in self.hit_counter.iter_mut() {
            let score = (*nb as f64) / ((self.nb_queries - *off) as f64);
            log::debug!("{:?}: {}", key, score);
        }
        let mut lfu: Vec<(I, f64)> = self
            .hit_counter
            .iter_mut()
            .map(|(k, (nb, off))| (k.clone(), (*nb as f64) / ((self.nb_queries - *off) as f64)))
            .collect();
        lfu.sort_by(|(_, a), (_, b)| a.total_cmp(b));
        for (key, _) in lfu.iter() {
            let content = self.cache.remove(key).unwrap();
            self.hit_counter.remove(key);
            let sz = content.len();
            log::debug!("Purging {:?} for the cache, freeing {} bytes", key, sz);
            self.current -= sz;
            if needed > sz {
                needed -= sz;
            } else {
                break;
            }
        }
    }
}

pub trait CacheElement: Clone {
    fn len(&self) -> usize;
}

pub struct Cache {
    posts: RwLock<CacheMap<u64, Post>>,
    post_md: RwLock<CacheMap<u64, PostMetadata>>,
    post_nav: RwLock<CacheMap<u64, String>>,
    post_page_rendered: RwLock<CacheMap<u64, String>>,
}

#[allow(dead_code)]
impl Cache {
    pub fn init(cfg: &Configuration) -> Cache {
        let size = cfg.cache.get_cache_size();
        log::debug!(
            "Cache sizes: {:?} for a total of {} bytes",
            size,
            cfg.cache.limit_size
        );
        Cache {
            posts: RwLock::new(CacheMap::empty(size[0])),
            post_nav: RwLock::new(CacheMap::empty(size[1])),
            post_page_rendered: RwLock::new(CacheMap::empty(size[2])),
            post_md: RwLock::new(CacheMap::empty(size[3])),
        }
    }

    pub fn get_post(&self, id: &u64) -> Option<Post> {
        self.posts.write().get_copy(id)
    }

    pub fn add_post(&self, post: Post) {
        self.posts.write().add(post.metadata.id, post);
    }

    pub fn add_post_nav(&self, id: u64, data: String) {
        self.post_nav.write().add(id, data);
    }

    pub fn get_post_nav(&self, id: &u64) -> Option<String> {
        self.post_nav.write().get_copy(id)
    }

    pub fn add_post_page(&self, id: u64, data: String) {
        self.post_page_rendered.write().add(id, data);
    }

    pub fn get_post_page(&self, id: &u64) -> Option<String> {
        self.post_page_rendered.write().get_copy(id)
    }

    pub fn add_post_md(&self, id: u64, data: PostMetadata) {
        self.post_md.write().add(id, data)
    }

    pub fn get_post_md(&self, id: &u64) -> Option<PostMetadata> {
        self.post_md.write().get_copy(id)
    }
}

impl CacheElement for String {
    fn len(&self) -> usize {
        self.len()
    }
}
