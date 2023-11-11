use actix_web::rt::task::JoinHandle;
use parking_lot::RwLock;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
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

    #[error("Render HTML from Markdown post")]
    HtmlFromMarkdown(#[from] mdtrans::Errcode),
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct LocalStorageConfig {
    pub refresh_duration: Duration,
    pub posts_dir: PathBuf,
    pub post_registry: PathBuf,
}

impl From<&Args> for LocalStorageConfig {
    fn from(args: &Args) -> Self {
        LocalStorageConfig {
            refresh_duration: Duration::from_secs(args.refresh_duration_secs),
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

#[allow(dead_code)]
#[derive(Clone)]
pub struct PostWatcher {
    thread: Arc<JoinHandle<()>>,
}

impl PostWatcher {
    pub fn init(
        cfg: &LocalStorageConfig,
        posts: PostsMap,
        series: SeriesList,
    ) -> Result<PostWatcher, LocalStorageError> {
        let thread = tokio::spawn(Self::serve(cfg.clone(), posts, series));
        Ok(PostWatcher {
            thread: Arc::new(thread),
        })
    }

    pub fn stop(&self) -> Result<(), LocalStorageError> {
        // TODO    Find a way to stop cleanly the loop
        self.thread.abort();
        Ok(())
    }

    #[allow(unused_mut)]
    async fn serve(mut cfg: LocalStorageConfig, posts: PostsMap, series: SeriesList) {
        let md_to_html = MarkdownRenderer::init();

        #[cfg(feature = "hot_reloading")]
        {
            cfg.refresh_duration = Duration::from_secs(3);
        }

        let mut discovered = vec![];
        loop {
            let tstart = std::time::Instant::now();
            let registry = load_registry(&cfg.post_registry).unwrap();

            *series.write() = registry.series;

            for (slug, mut metadata) in registry.posts {
                if discovered.contains(&slug) {
                    continue;
                }
                let Some(fpath) = registry.post_contents_path.get(&slug) else {
                    log::warn!("Unable to find blog post {slug} in post_contents_path list");
                    continue;
                };
                let post_markdown = std::fs::read_to_string(cfg.posts_dir.join(fpath))
                    .expect("Unable to read post {fpath:?}");
                let (content, post_nav) = md_to_html.render(post_markdown, &metadata).unwrap();
                metadata.compute_id();
                let id = metadata.id;
                let post = Post {
                    metadata,
                    content,
                    post_nav,
                };
                discovered.push(slug);
                posts.write().insert(id, post);
            }

            tokio::time::sleep(cfg.refresh_duration.saturating_sub(tstart.elapsed())).await
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct PostsRegistry {
    series: HashMap<String, SerieMetadata>,
    posts: HashMap<String, PostMetadata>,
    post_contents_path: HashMap<String, PathBuf>,
}

type PostsMap = Arc<RwLock<BTreeMap<u64, Post>>>;
type SeriesList = Arc<RwLock<HashMap<String, SerieMetadata>>>;

#[derive(Clone)]
pub struct LocalStorage {
    post_watcher: PostWatcher,
    posts: PostsMap,
    series: SeriesList,
}

fn load_registry(fpath: &PathBuf) -> Result<PostsRegistry, LocalStorageError> {
    let reg_str = std::fs::read_to_string(fpath)?;
    let new_registry = toml::from_str::<PostsRegistry>(reg_str.as_str())?;
    Ok(new_registry)
}

impl StorageTrait for LocalStorage {
    type Error = LocalStorageError;

    fn init(config: &Arc<Configuration>) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let storage_config = config.storage_cfg.clone();
        let posts = Arc::new(RwLock::new(BTreeMap::new()));
        let series = Arc::new(RwLock::new(HashMap::new()));
        let post_watcher = PostWatcher::init(&storage_config, posts.clone(), series.clone())?;
        Ok(LocalStorage {
            post_watcher,
            posts,
            series,
        })
    }

    fn clean_exit(self) -> Result<(), Errcode> {
        self.post_watcher.stop()?;
        Ok(())
    }

    fn query_post_metadata(&self, query: StorageQuery) -> Result<Vec<PostMetadata>, Self::Error> {
        Ok({
            let posts = self.posts.read();
            let mut all_md = posts
                .values()
                .filter(|p| p.metadata.filter(&query.post_filter))
                .map(|p| &p.metadata)
                .collect::<Vec<&PostMetadata>>();
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
            all_md
                .into_iter()
                .skip(query.offset)
                .take(limit)
                .cloned()
                .map(|mut md| {
                    if let Some(ref slug) = md.serie {
                        md.serie_title = self.series.read().get(slug).map(|t| t.title.clone());
                    }
                    md
                })
                .collect()
        })
    }

    fn query_serie(&self, query: StorageQuery) -> Result<Vec<SerieMetadata>, Self::Error> {
        Ok({
            let series = self.series.read();
            let mut all_series: Vec<(&String, &SerieMetadata)> = series
                .iter()
                .filter(|(slug, _)| {
                    if let Some(ref qslug) = query.serie_slug {
                        qslug == *slug
                    } else {
                        true
                    }
                })
                .collect();

            all_series.sort_by(|(_, a), (_, b)| {
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

            all_series
                .into_iter()
                .skip(query.offset)
                .take(limit)
                .map(|(slug, md)| {
                    let mut md = md.clone();
                    md.slug = slug.clone();
                    md
                })
                .collect()
        })
    }

    fn query_category(&self, query: StorageQuery) -> Result<Vec<String>, Self::Error> {
        Ok({
            let posts = self.posts.read();
            let all_categories: HashSet<&String> = posts
                .values()
                .filter_map(|p| p.metadata.category.as_ref())
                .collect();
            let mut all_categories = all_categories.into_iter().collect::<Vec<&String>>();

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
            all_categories
                .into_iter()
                .skip(query.offset)
                .take(limit)
                .cloned()
                .collect()
        })
    }

    fn get_post_content(&self, id: u64) -> Result<Option<Post>, Self::Error> {
        Ok(self.posts.read().get(&id).cloned())
    }
}
