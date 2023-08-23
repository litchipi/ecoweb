use std::sync::Arc;

use crate::cache::Cache;
use crate::config::Configuration;
use crate::errors::Errcode;
use crate::post::{Post, PostMetadata};
use crate::render::nav::NavRenderer;

use super::{
    storage::{Storage, StorageQuery, StorageTrait},
    PostFilter,
};

pub struct PostLoader {
    #[allow(dead_code)]
    cache: Arc<Cache>,
    config: Arc<Configuration>,
    storage: Storage,
    nav_render: NavRenderer,
}

impl PostLoader {
    pub fn init(config: Arc<Configuration>, cache: Arc<Cache>, storage: Storage) -> PostLoader {
        let nav_render = NavRenderer::init();
        PostLoader {
            config,
            cache,
            storage,
            nav_render,
        }
    }

    /// Get the content of a post with its ID, stores to cache if possible
    pub fn get(&self, id: u64) -> Result<Option<(Post, String)>, Errcode> {
        let post = if let Some(post) = self.cache.get_post(&id) {
            post
        } else {
            let Some(post) = self.storage.get_post_content(id)? else {
                return Ok(None);
            };
            self.cache.add_post(post.clone());
            post
        };

        let nav = if let Some(nav) = self.cache.get_post_nav(&id) {
            nav
        } else {
            let res = self.nav_render.render(&post.content);
            self.cache.add_post_nav(id, res.clone());
            res
        };

        Ok(Some((post, nav)))
    }

    /// Get all the most recent posts
    pub fn get_recent(
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
        Ok(self.storage.query_post(query)?)
    }

    /// List of posts that fits into a given serie
    pub fn list_posts_serie(
        &self,
        serie: String,
        mut add_filter: Vec<PostFilter>,
    ) -> Result<Vec<PostMetadata>, Errcode> {
        add_filter.push(PostFilter::Serie(serie));
        let mut query = StorageQuery::empty();
        query.reverse_order = false;
        query.post_filter = PostFilter::Combine(add_filter);
        Ok(self.storage.query_post(query)?)
    }

    /// List of posts that fits into a given category
    pub fn list_posts_category(
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

        Ok(self.storage.query_post(query)?)
    }
}
