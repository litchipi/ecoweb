use crate::cache::Cache;
use crate::config::Config;

mod context;
mod data;
mod query;
mod backend;

pub use context::ContextQuery;
pub use data::{PageMetadata, StorageData};
pub use query::StorageQuery;
use backend::{StorageBackend, LocalStorage};

pub type StorageSlug = String;

#[cfg(feature = "storage-local")]
pub type Storage = StorageImpl<LocalStorage>;

pub struct StorageImpl<T: StorageBackend> {
    cache: Cache<StorageQuery, StorageData>,
    backend: T,
}

impl<T: StorageBackend> StorageImpl<T> {
    pub fn init(config: &Config) -> StorageImpl<T> {
        StorageImpl {
            cache: Cache::empty(),
            backend: T::init(config),
        }
    }

    pub async fn query(&self, qry: &StorageQuery) -> StorageData {
        // TODO    Calls the "has_changed" functions of StorageBackend
        // If not changed, get from cache
        // Else, get from backend, and add to cache
        self.backend.query(qry).await
    }

    pub async fn has_changed(&self, qry: &StorageQuery) -> bool {
        self.backend.has_changed(qry).await
    }

    // TODO    Add a way to save data into the storage as well
}
