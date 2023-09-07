use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;

use serde::{Deserialize, Serialize};

use crate::config::Configuration;
use crate::errors::Errcode;
use crate::post::{Post, PostMetadata, SerieMetadata};
use crate::render::markdown::MarkdownRenderer;
use crate::Args;

use super::{StorageQuery, StorageTrait};

#[derive(Error, Debug)]
pub enum LocalStorageError {
    #[error("Failed to decode posts registry")]
    RegistryLoadError(#[from] toml::de::Error),

    #[error("Input / Output error")]
    IOError(#[from] std::io::Error),
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct LocalStorageConfig {
    pub posts_dir: PathBuf,
    pub post_registry: PathBuf,
}

impl From<&Args> for LocalStorageConfig {
    fn from(args: &Args) -> Self {
        LocalStorageConfig {
            post_registry: args.posts_registry.clone(),
            posts_dir: args.posts_dir.clone(),
        }
    }
}

impl LocalStorageConfig {
    pub fn validate(&self) -> Result<(), Errcode> {
        if !self.posts_dir.exists() {
            return Err(Errcode::PathDoesntExist(
                "posts dir",
                self.posts_dir.clone(),
            ));
        }
        if !self.post_registry.exists() {
            return Err(Errcode::PathDoesntExist(
                "posts list",
                self.post_registry.clone(),
            ));
        }
        Ok(())
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct PostsRegistry {
    cache_duration: u64, // In seconds

    series: HashMap<String, SerieMetadata>,
    posts: HashMap<String, PostMetadata>,
    post_contents_path: HashMap<String, PathBuf>,
}

#[derive(Clone)]
pub struct LocalStorage {
    config: LocalStorageConfig,
    md_render: MarkdownRenderer,
    registry: Arc<RwLock<PostsRegistry>>,
    last_updated: Arc<RwLock<Instant>>,
}

impl LocalStorage {
    // TODO    Find a cleaner way to check last update registry cache
    pub fn update_registry_if_needed(&self) {
        #[cfg(feature = "hot_reloading")]
        {
            self.registry.write().cache_duration = 0;
        }

        let updated = if self.last_updated.read().elapsed()
            > Duration::from_secs(self.registry.read().cache_duration)
        {
            let Ok(reg_str) = std::fs::read_to_string(&self.config.post_registry) else {
                return;
            };
            let Ok(mut new_registry) = toml::from_str::<PostsRegistry>(
                reg_str.as_str()
            ) else {
                return;
            };
            new_registry.posts.iter_mut().for_each(|(_, md)| {
                if let Some(ref slug) = md.serie {
                    md.serie_title = new_registry
                        .series
                        .get(slug)
                        .map(|serie_md| serie_md.title.clone());
                }
                md.compute_id()
            });
            *self.registry.write() = new_registry;
            true
        } else {
            false
        };
        if updated {
            *self.last_updated.write() = Instant::now();
        }
    }
}

impl StorageTrait for LocalStorage {
    type Error = LocalStorageError;

    fn init(config: &Arc<Configuration>) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let storage_config = config.storage_cfg.clone();
        let registry =
            toml::from_str(std::fs::read_to_string(&storage_config.post_registry)?.as_str())?;
        Ok(LocalStorage {
            md_render: MarkdownRenderer::init(config),
            config: storage_config,
            registry: Arc::new(RwLock::new(registry)),
            last_updated: Arc::new(RwLock::new(
                Instant::now() - Duration::from_secs(60 * 60 * 24),
            )),
        })
    }

    fn query_post(&self, query: StorageQuery) -> Result<Vec<PostMetadata>, Self::Error> {
        self.update_registry_if_needed();
        let mut all_md: Vec<PostMetadata> = self
            .registry
            .read()
            .posts
            .iter()
            .filter(|(_, md)| md.filter(&query.post_filter))
            .map(|(_, md)| md)
            .cloned()
            .collect();
        all_md.sort_by(|a, b| {
            let ord = a.date.cmp(&b.date);
            if query.reverse_order {
                ord.reverse()
            } else {
                ord
            }
        });
        let limit = if query.limit == 0 {
            all_md.len()
        } else {
            query.limit
        };
        Ok(all_md.into_iter().skip(query.offset).take(limit).collect())
    }

    fn query_serie(&self, query: StorageQuery) -> Result<Vec<SerieMetadata>, Self::Error> {
        self.update_registry_if_needed();
        let mut all_series: Vec<SerieMetadata> = self
            .registry
            .read()
            .series
            .iter()
            .filter(|(slug, _)| {
                if let Some(ref qslug) = query.serie_slug {
                    qslug == *slug
                } else {
                    true
                }
            })
            .map(|(slug, md)| {
                let mut md = md.clone();
                md.slug = slug.clone();
                md
            })
            .collect();

        all_series.sort_by(|a, b| {
            let ord = a.end_date.cmp(&b.end_date);
            if query.reverse_order {
                ord.reverse()
            } else {
                ord
            }
        });
        let limit = if query.limit == 0 {
            all_series.len()
        } else {
            query.limit
        };
        Ok(all_series
            .into_iter()
            .skip(query.offset)
            .take(limit)
            .collect())
    }

    fn query_category(&self, query: StorageQuery) -> Result<Vec<String>, Self::Error> {
        self.update_registry_if_needed();
        let all_categories: HashSet<String> = self
            .registry
            .read()
            .posts
            .iter()
            .filter_map(|(_, md)| md.category.clone())
            .collect();

        let mut all_categories = all_categories.into_iter().collect::<Vec<String>>();
        all_categories.sort_by(|a, b| {
            let ord = a.cmp(b);
            if query.reverse_order {
                ord.reverse()
            } else {
                ord
            }
        });
        let limit = if query.limit == 0 {
            all_categories.len()
        } else {
            query.limit
        };
        Ok(all_categories
            .into_iter()
            .skip(query.offset)
            .take(limit)
            .collect())
    }

    fn get_post_content(&self, id: u64) -> Result<Option<Post>, Self::Error> {
        self.update_registry_if_needed();

        let Some((slug, mut metadata)) = self.registry.read().posts
            .iter()
            .filter_map(|(slug, md)| if md.id == id {
                Some((slug, md))
            } else {
                None
            }).last().map(|(slug, md)| (slug.clone(), md.clone())) else {
            return Ok(None);
        };

        let fpath = if let Some(path) = self.registry.read().post_contents_path.get(&slug) {
            path.clone()
        } else {
            return Ok(None);
        };
        let content = std::fs::read_to_string(self.config.posts_dir.join(fpath))?;
        let html = self.md_render.render(content, &metadata);

        metadata.compute_checksum(&html);
        metadata.compute_id();

        Ok(Some(Post {
            metadata,
            content: html,
        }))
    }
}
