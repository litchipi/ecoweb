use std::sync::Arc;

use crate::{
    config::Configuration,
    errors::Errcode,
    post::{Post, PostMetadata, SerieMetadata},
};

use super::PostFilter;

pub mod local_storage;

#[cfg(feature = "local_storage")]
pub type Storage = local_storage::LocalStorage;

pub trait StorageTrait {
    type Error: Into<Errcode>;
    fn init(config: &Arc<Configuration>) -> Result<Self, Self::Error>
    where
        Self: Sized;
    fn query_post(&self, query: StorageQuery) -> Result<Vec<PostMetadata>, Self::Error>;
    fn query_serie(&self, query: StorageQuery) -> Result<Vec<SerieMetadata>, Self::Error>;
    fn query_category(&self, query: StorageQuery) -> Result<Vec<String>, Self::Error>;
    fn get_post_content(&self, id: u64) -> Result<Option<Post>, Self::Error>;
}

#[derive(Debug)]
pub struct StorageQuery {
    pub limit: usize, // 0 = no limit
    pub offset: usize,
    pub reverse_order: bool,

    // Query on post
    pub post_filter: PostFilter,

    // Query on series
    pub serie_slug: Option<String>,
}

impl StorageQuery {
    pub fn empty() -> StorageQuery {
        StorageQuery {
            limit: 0,
            offset: 0,
            reverse_order: false,
            post_filter: PostFilter::NoFilter,
            serie_slug: None,
        }
    }
}
