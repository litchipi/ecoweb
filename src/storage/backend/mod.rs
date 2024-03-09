use crate::config::Config;

use super::{StorageData, StorageQuery};

mod local;

pub use local::LocalStorage;

pub trait StorageBackend {
    fn init(config: &Config) -> Self where Self: Sized;
    async fn has_changed(&self, qry: &StorageQuery) -> bool;
    async fn query(&self, qry: &StorageQuery) -> StorageData;
}
