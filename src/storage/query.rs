use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

use serde::{Deserialize, Serialize};

use super::context::{MetadataFilter, MetadataQuery};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct QueryListOptions {
    #[serde(default)]
    limit: usize,
    #[serde(default)]
    sort_by: Option<String>,
    #[serde(default)]
    rev_sort: bool,
}

#[repr(u8)]
#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize, Clone)]
/// All the methods that a storage have to implement in order to work
pub enum StorageQueryMethod {
    #[default]
    NoOp = 0,
    ContentNoId,
    ContentNumId(u64),
    ContentSlug(String),
    RecentPages,
    GetSimilarPages(MetadataFilter),
    PageTemplate(String),
    BaseTemplates,
    StaticFile(String),
    QueryMetadata(MetadataFilter, MetadataQuery),
}

impl StorageQueryMethod {
    pub fn build_query<T: ToString + ?Sized>(self, slug: &T) -> StorageQuery {
        let mut qry = StorageQuery {
            storage_slug: slug.to_string(),
            method: self,
            ..Default::default()
        };
        qry.update_key();
        qry
    }
}

#[derive(Debug, Default, Eq, Clone)]
pub struct StorageQuery {
    key: u64,
    pub storage_slug: String,
    pub method: StorageQueryMethod,
    pub limit: usize,
    pub lang_pref: Option<Vec<String>>,
    pub sort_by: Option<(String, bool)>,
}

impl std::hash::Hash for StorageQuery {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.key)
    }
}

impl std::cmp::PartialEq for StorageQuery {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl StorageQuery {
    pub fn query_metadata(slug: &String, filter: MetadataFilter, qry: MetadataQuery) -> StorageQuery {
        StorageQueryMethod::QueryMetadata(filter, qry).build_query(slug)
    }
    pub fn similar_pages(slug: &String, keys: MetadataFilter, opts: &QueryListOptions) -> StorageQuery {
        let mut qry = StorageQueryMethod::GetSimilarPages(keys).build_query(slug);
        qry.list_opts(opts);
        qry
    }
    pub fn static_file(fname: String) -> StorageQuery {
        StorageQueryMethod::StaticFile(fname).build_query("static")
    }
    pub fn base_templates() -> StorageQuery {
        StorageQueryMethod::BaseTemplates.build_query("templates")
    }
    pub fn template(slug: String) -> StorageQuery {
        StorageQueryMethod::PageTemplate(slug).build_query("templates")
    }
    pub fn recent_pages(slug: &String, opts: &QueryListOptions) -> StorageQuery {
        let mut qry = StorageQueryMethod::RecentPages.build_query(slug);
        qry.list_opts(opts);
        qry
    }
    pub fn content(storage: &String, id: Option<u64>, slug: Option<String>) -> StorageQuery {
        let method = if let Some(id) = id {
            StorageQueryMethod::ContentNumId(id)
        } else {
            if let Some(slug) = slug {
                StorageQueryMethod::ContentSlug(slug)
            } else {
                StorageQueryMethod::ContentNoId
            }
        };
        method.build_query(storage)
    }

    pub fn update_key(&mut self) {
        let mut s = DefaultHasher::new();
        s.write(self.storage_slug.as_bytes());
        // s.write_u8(self.method as u8);    // TODO Make this work
        match self.method {
            StorageQueryMethod::NoOp => s.write_u8(0),
            StorageQueryMethod::ContentNoId => s.write_u8(1),
            StorageQueryMethod::ContentNumId(id) => {
                s.write_u8(2);
                s.write_u64(id);
            }
            StorageQueryMethod::RecentPages => s.write_u8(3),
            StorageQueryMethod::PageTemplate(ref n) => {
                s.write_u8(4);
                s.write(n.as_bytes());
            }
            StorageQueryMethod::BaseTemplates => s.write_u8(5),
            StorageQueryMethod::StaticFile(ref n) => {
                s.write_u8(6);
                s.write(n.as_bytes());
            }
            StorageQueryMethod::GetSimilarPages((ref keys, ref v)) => {
                s.write_u8(7);
                for k in keys {
                    s.write(k.as_bytes());
                }
                s.write(format!("{v:?}").as_bytes());
            }
            StorageQueryMethod::ContentSlug(ref slug) => {
                s.write_u8(8);
                s.write(slug.as_bytes());
            }
            StorageQueryMethod::QueryMetadata(ref filter, ref qry) => {
                s.write_u8(9);
                let (keys, val) = filter;
                for f in keys {
                    s.write(f.as_bytes());
                }
                s.write(format!("{val:?}").as_bytes());
                for q in qry {
                    s.write(q.as_bytes());
                }
            }
        }
        s.write_usize(self.limit);
        self.key = s.finish();
    }

    pub fn set_lang(&mut self, lang: Vec<String>) {
        self.lang_pref = Some(lang);
    }

    pub fn list_opts(&mut self, opts: &QueryListOptions) {
        self.limit = opts.limit;
        self.sort_by = opts.sort_by.clone().map(|s| (s, opts.rev_sort));
        self.update_key();
    }
}
