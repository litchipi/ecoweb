use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};

use crate::config::Configuration;
use crate::errors::Errcode;
use crate::post::{Post, PostMetadata, SerieMetadata};
use crate::render::markdown::MarkdownRenderer;
use crate::Args;

use super::{StorageQuery, StorageTrait};

#[derive(Debug)]
pub enum LocalStorageError {
    GlobError(glob::GlobError),
    GlobPattern(glob::PatternError),
    MetadataParsing(Box<Errcode>),
    IoError(std::io::Error),
    SerieLoadFailed(toml::de::Error),
    SerieNotFound(String),
}

impl LocalStorageError {
    pub fn get_err_str(&self) -> String {
        match self {
            LocalStorageError::MetadataParsing(e) => format!("Unable to parse metadata: {e:?}"),
            e => format!("{e:?}"),
        }
    }
}

impl From<std::io::Error> for LocalStorageError {
    fn from(value: std::io::Error) -> Self {
        LocalStorageError::IoError(value)
    }
}

impl From<toml::de::Error> for LocalStorageError {
    fn from(value: toml::de::Error) -> Self {
        LocalStorageError::SerieLoadFailed(value)
    }
}

impl From<glob::PatternError> for LocalStorageError {
    fn from(value: glob::PatternError) -> Self {
        LocalStorageError::GlobPattern(value)
    }
}

impl From<glob::GlobError> for LocalStorageError {
    fn from(value: glob::GlobError) -> Self {
        LocalStorageError::GlobError(value)
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct LocalStorageConfig {
    pub posts_dir: PathBuf,
    pub series_list: PathBuf,
}

impl From<&Args> for LocalStorageConfig {
    fn from(args: &Args) -> Self {
        LocalStorageConfig {
            posts_dir: args.posts_dir.clone(),
            series_list: args.series_list.clone(),
        }
    }
}

impl LocalStorageConfig {
    pub fn validate(&self) -> Result<(), Errcode> {
        if !self.posts_dir.exists() {
            return Err(Errcode::PathDoesntExist("posts", self.posts_dir.clone()));
        }
        if !self.series_list.exists() {
            return Err(Errcode::PathDoesntExist(
                "series.toml",
                self.series_list.clone(),
            ));
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct LocalStorage {
    config: LocalStorageConfig,
    path_cache: Arc<RwLock<BTreeMap<u64, PathBuf>>>,
    series_cache: Arc<RwLock<HashMap<String, SerieMetadata>>>,
    md_render: MarkdownRenderer,
}

impl LocalStorage {
    fn search_post_fpath(&self, id: u64) -> Result<Option<PathBuf>, LocalStorageError> {
        if let Some(path) = self.path_cache.read().unwrap().get(&id) {
            if path.exists() {
                return Ok(Some(path.clone()));
            }
        }

        let all_paths =
            glob::glob(format!("{}/**/*.md", self.config.posts_dir.to_str().unwrap()).as_str())?;
        for fres in all_paths.into_iter() {
            let f = fres?;
            let post_metadata = PostMetadata::read_from_file(&f.with_extension("json"))
                .map_err(|e| LocalStorageError::MetadataParsing(Box::new(e)))?;
            if post_metadata.id == id {
                log::debug!("Found post {}", post_metadata.id);
                self.path_cache.write().unwrap().insert(id, f.clone());
                return Ok(Some(f));
            }
        }
        Ok(None)
    }

    fn update_series_list(&self) -> Result<(), LocalStorageError> {
        let strdata = std::fs::read_to_string(&self.config.series_list)?;
        let all_series: HashMap<String, SerieMetadata> = toml::from_str(&strdata)?;

        let mut series = self
            .series_cache
            .write()
            .expect("write series update_series_list");
        *series = HashMap::new();
        for (slug, mut md) in all_series.into_iter() {
            md.slug = slug.clone();
            series.insert(slug, md);
        }
        Ok(())
    }

    fn try_get_serie(&self, md: &mut PostMetadata) -> Result<(), LocalStorageError> {
        if let Some(ref slug) = md.serie {
            let title_opt = self
                .series_cache
                .read()
                .expect("try_get_serie read series_cache")
                .get(slug)
                .map(|md| md.title.clone());
            if let Some(title) = title_opt {
                md.serie_title = Some(title);
            } else {
                self.update_series_list()?;
                match self
                    .series_cache
                    .read()
                    .expect("try_get_serie read series_cache")
                    .get(slug)
                {
                    Some(serie_md) => md.serie_title = Some(serie_md.title.clone()),
                    None => return Err(LocalStorageError::SerieNotFound(slug.clone())),
                }
            }
        }
        Ok(())
    }
}

impl StorageTrait for LocalStorage {
    type Error = LocalStorageError;

    fn init(config: &Arc<Configuration>) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(LocalStorage {
            md_render: MarkdownRenderer::init(config),
            config: config.storage_cfg.clone(),
            series_cache: Arc::new(RwLock::new(HashMap::new())),
            path_cache: Arc::new(RwLock::new(BTreeMap::new())),
        })
    }

    fn query_post(&self, query: StorageQuery) -> Result<Vec<PostMetadata>, Self::Error> {
        let mut res = vec![];
        let all_paths =
            glob::glob(format!("{}/**/*.json", self.config.posts_dir.to_str().unwrap()).as_str())?;

        for fres in all_paths.into_iter() {
            let f = fres?;
            let mut post_metadata = PostMetadata::read_from_file(&f)
                .map_err(|e| LocalStorageError::MetadataParsing(Box::new(e)))?;
            if post_metadata.filter(&query.post_filter) {
                self.try_get_serie(&mut post_metadata)?;
                res.push(post_metadata);
            }
        }
        res.sort_by(|a, b| {
            let ord = a.date.cmp(&b.date);
            if query.reverse_order {
                ord.reverse()
            } else {
                ord
            }
        });
        let limit = if query.limit == 0 {
            res.len()
        } else {
            query.limit
        };
        Ok(res.into_iter().skip(query.offset).take(limit).collect())
    }

    fn query_serie(&self, query: StorageQuery) -> Result<Vec<SerieMetadata>, Self::Error> {
        self.update_series_list()?;
        let mut res: Vec<SerieMetadata> = self
            .series_cache
            .read()
            .expect("read series_cache query_serie")
            .values()
            .filter(|md| {
                if let Some(ref slug) = query.serie_slug {
                    slug == &md.slug
                } else {
                    true
                }
            })
            .cloned()
            .collect();
        res.sort_by(|a, b| {
            let ord = a.end_date.cmp(&b.end_date);
            if query.reverse_order {
                ord.reverse()
            } else {
                ord
            }
        });
        let limit = if query.limit == 0 {
            res.len()
        } else {
            query.limit
        };
        Ok(res.into_iter().skip(query.offset).take(limit).collect())
    }

    fn query_category(&self, query: StorageQuery) -> Result<Vec<String>, Self::Error> {
        let mut res = vec![];
        let all_paths =
            glob::glob(format!("{}/**/*.json", self.config.posts_dir.to_str().unwrap()).as_str())?;

        for fres in all_paths.into_iter() {
            let f = fres?;
            let post_metadata = PostMetadata::read_from_file(&f)
                .map_err(|e| LocalStorageError::MetadataParsing(Box::new(e)))?;
            if let Some(ref cat) = post_metadata.category {
                if !res.contains(cat) {
                    res.push(cat.clone());
                }
            }
        }
        res.sort_by(|a, b| {
            let ord = a.cmp(b);
            if query.reverse_order {
                ord.reverse()
            } else {
                ord
            }
        });
        let limit = if query.limit == 0 {
            res.len()
        } else {
            query.limit
        };
        Ok(res.into_iter().skip(query.offset).take(limit).collect())
    }

    fn get_post_content(&self, id: u64) -> Result<Option<Post>, Self::Error> {
        let Some(fpath) = self.search_post_fpath(id)? else {
            return Ok(None);
        };
        let content = std::fs::read_to_string(&fpath)?;
        let mut metadata = PostMetadata::read_from_file(&fpath.with_extension("json"))
            .map_err(|e| LocalStorageError::MetadataParsing(Box::new(e)))?;

        let html = self.md_render.render(content, &metadata);

        metadata.compute_checksum(&html);
        metadata.compute_id();

        Ok(Some(Post {
            metadata,
            content: html,
        }))
    }
}
