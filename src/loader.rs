use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::config::Configuration;
use crate::errors::Errcode;
use crate::post::SerieMetadata;

pub mod storage;
use storage::Storage;

mod post_loader;

use self::storage::{StorageQuery, StorageTrait};

#[allow(dead_code)]
pub struct Loader {
    storage: Storage,
    pub posts: Arc<post_loader::PostLoader>,
}

impl Loader {
    pub fn init(config: Arc<Configuration>) -> Result<Loader, Errcode> {
        let storage = Storage::init(&config)?;
        Ok(Loader {
            posts: Arc::new(post_loader::PostLoader::init(config, storage.clone())),
            storage,
        })
    }

    #[allow(dead_code)]
    pub fn reload(&self) -> Result<(), Errcode> {
        self.storage.reload()?;
        Ok(())
    }

    pub fn clean_exit(self) -> Result<(), Errcode> {
        self.storage.clean_exit()?;
        Ok(())
    }

    pub fn get_all_categories(&self) -> Result<Vec<String>, Errcode> {
        let query = StorageQuery::empty();
        Ok(self.storage.query_category(query)?)
    }

    pub fn get_all_series(&self) -> Result<Vec<SerieMetadata>, Errcode> {
        let mut query = StorageQuery::empty();
        query.reverse_order = true; // Get newly finished first
        Ok(self.storage.query_serie(query)?)
    }

    pub fn get_serie_md(&self, slug: String) -> Result<Option<SerieMetadata>, Errcode> {
        let mut query = StorageQuery::empty();
        query.limit = 1;
        query.serie_slug = Some(slug);
        Ok(self.storage.query_serie(query)?.into_iter().next())
    }
}

#[derive(Debug)]
pub enum PostFilter {
    NoFilter,
    NoSerie,
    Serie(String),
    Category(String),
    Combine(Vec<PostFilter>),
    DifferentThan(u64),
    ContainsTag(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoadingLimits {
    recent_posts: usize,
    categories: usize,
    series: usize,
    plain_posts_list: usize,
}

impl Default for LoadingLimits {
    fn default() -> Self {
        LoadingLimits {
            recent_posts: 4,
            categories: 5,
            series: 5,
            plain_posts_list: 15,
        }
    }
}
