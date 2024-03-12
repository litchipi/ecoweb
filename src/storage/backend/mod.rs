use actix_web::HttpResponseBuilder;
use serde::{de::DeserializeOwned, Serialize};

use crate::config::Config;

use super::{StorageData, StorageQuery};

pub mod local;

pub trait StorageBackend {
    type Error: Into<HttpResponseBuilder> + Clone + Serialize + DeserializeOwned + std::fmt::Debug;
    fn init(config: &Config) -> Result<Self, Self::Error>
    where
        Self: Sized;
    async fn has_changed(&self, qry: &StorageQuery) -> bool;
    async fn query(&self, qry: StorageQuery) -> StorageData;
}
