use crate::cache::Cache;
use crate::config::Config;

mod context;
mod data;
mod query;
pub mod backend;

pub use context::ContextQuery;
pub use data::StorageData;
pub use query::StorageQuery;
use backend::StorageBackend;

pub type StorageSlug = String;

#[cfg(feature = "storage-local")]
pub type Storage = StorageImpl<backend::local::LocalStorage>;

pub struct StorageImpl<T: StorageBackend> {
    cache: Cache<StorageQuery, StorageData>,
    backend: T,
}

impl<T: StorageBackend> StorageImpl<T> {
    pub fn init(config: &Config) -> StorageImpl<T> {
        StorageImpl {
            cache: Cache::empty(1024), // TODO Get from config
            backend: T::init(config),
        }
    }

    pub async fn query(&self, qry: StorageQuery) -> StorageData {
        if let Some(data) = self.cache.get(&qry) {
            if !self.has_changed(&qry).await {
                return data;
            }
        }
        let data = self.backend.query(qry.clone()).await;
        self.cache.add(qry, data.clone());
        data
    }

    pub async fn has_changed(&self, qry: &StorageQuery) -> bool {
        self.backend.has_changed(qry).await
    }

    // TODO    Add a way to save data into the storage as well
}
