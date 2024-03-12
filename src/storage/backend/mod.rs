use crate::config::Config;

use super::{StorageData, StorageQuery};

pub mod local;

pub trait StorageBackend {
    fn init(config: &Config) -> Self
    where
        Self: Sized;
    async fn has_changed(&self, qry: &StorageQuery) -> bool;
    async fn query(&self, qry: StorageQuery) -> StorageData;
}
