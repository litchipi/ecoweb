use std::sync::Arc;

use crate::config::Configuration;
use crate::errors::Errcode;
use crate::post::{Post, PostMetadata};

use super::{
    storage::{Storage, StorageQuery, StorageTrait},
    PostFilter,
};

pub struct PostLoader {
    config: Arc<Configuration>,
    storage: Storage,
}

impl PostLoader {
    pub fn init(config: Arc<Configuration>, storage: Storage) -> PostLoader {
        PostLoader { config, storage }
    }

    /// Get the content of a post with its ID, stores to cache if possible
    pub async fn get(&self, id: u64) -> Result<Option<Post>, Errcode> {
        let Some(post) = self.storage.get_post_content(id).await? else {
            return Ok(None);
        };
        Ok(Some(post))
    }

    /// Get all the most recent posts
    pub async fn get_recent(
        &self,
        filter: PostFilter,
        reverse_order: bool,
        limit: Option<usize>,
    ) -> Result<Vec<PostMetadata>, Errcode> {
        let mut query = StorageQuery::empty();
        query.offset = 0;
        query.limit = if let Some(l) = limit {
            l
        } else {
            self.config.limits.plain_posts_list
        };
        query.reverse_order = reverse_order;
        query.post_filter = filter;
        Ok(self.storage.query_post_metadata(query).await?)
    }

    /// List of posts that fits into a given serie
    pub async fn list_posts_serie(
        &self,
        serie: String,
        mut add_filter: Vec<PostFilter>,
    ) -> Result<Vec<PostMetadata>, Errcode> {
        add_filter.push(PostFilter::Serie(serie));
        let mut query = StorageQuery::empty();
        query.reverse_order = false;
        query.post_filter = PostFilter::Combine(add_filter);
        Ok(self.storage.query_post_metadata(query).await?)
    }

    /// List of posts that fits into a given category
    pub async fn list_posts_category(
        &self,
        category: String,
        mut add_filter: Vec<PostFilter>,
    ) -> Result<Vec<PostMetadata>, Errcode> {
        add_filter.push(PostFilter::Category(category));

        let mut query = StorageQuery::empty();
        query.offset = 0;
        query.limit = self.config.limits.plain_posts_list;
        query.reverse_order = true;
        query.post_filter = PostFilter::Combine(add_filter);

        Ok(self.storage.query_post_metadata(query).await?)
    }
}
