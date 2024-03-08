use crate::config::Config;
use crate::cache::Cache;

mod query;
mod data;
mod context;

pub use query::StorageQuery;
pub use data::{StorageData, PageMetadata};
pub use context::ContextQuery;

pub type StorageSlug = String;

pub trait StorageBackend {
    // TODO    Add functions for storage backend trait
    fn has_changed(&self, qry: &StorageQuery) -> bool;
}

pub struct Storage {
    cache: Cache<StorageQuery, StorageData>,
    // TODO    Implement this for different kinds of storage backends
}

impl Storage {
    pub fn init(config: &Config) -> Storage {
        Storage { cache: Cache::empty() }
    }

    pub fn query(&self, qry: &StorageQuery) -> StorageData {
        // TODO    Calls the "has_changed" functions of StorageBackend
        // If not changed, get from cache
        // Else, get from backend, and add to cache
        StorageData::RecentPages(vec![])
    }

    // TODO    Add a way to save data into the storage as well
}
