use serde::{Deserialize, Serialize};

use crate::{config::Config, dispatch::UrlBuildMethod};

pub type StorageSlug = String;

#[derive(Serialize, Deserialize, Clone)]
pub enum ContextQuery {
    RecentPages(String, usize),
}

impl ContextQuery {
    pub fn insert_context(&self, ldr: &Storage, ctxt: &mut Context) {
        match self {
            ContextQuery::RecentPages(ptype, nb) => {
                let val = ldr.query(&StorageQuery::recent_pages(ptype.clone(), *nb));
            },
        }
    }
}

#[derive(Default, Hash, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct StorageQuery {
    pub lang_pref: Option<Vec<String>>,
}

impl StorageQuery {
    pub fn recent_pages(ptype: String, nb: usize) -> StorageQuery {
        StorageQuery::default()
    }

    pub fn content_from(method: &UrlBuildMethod, req: &RequestParams) -> StorageQuery {
        match method {
            UrlBuildMethod::ContentId => {
                StorageQuery::default()
            },
            UrlBuildMethod::FromMetadata(key) => StorageQuery::default()
        }
    }
}

pub enum StorageData {
    PageContent { metadata: PageMetadata, body: String },
    RecentPages(Vec<PageMetadata>),
}

#[derive(Serialize, Deserialize)]
pub struct PageMetadata {
    pub add_context: Vec<ContextQuery>,
}

pub struct Storage {
    
}

impl Storage {
    pub fn init(config: &Config) -> Storage {
        Storage {  }
    }

    pub fn query_cache(&self, qry: &StorageQuery) -> (PageMetadata, String) {
        // Query a post from cache, it knows it's recorded in it
        todo!();
    }

    pub fn has_changed(&self, qry: &StorageQuery) -> bool {
        // Returns if a post has been updated since last read
        false
    }

    pub fn query(&self, qry: &StorageQuery) -> StorageData {
        todo!();
    }
}
