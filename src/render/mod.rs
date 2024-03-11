use tera::Context;

use crate::cache::Cache;
use crate::config::Config;
use crate::page::PageType;
use crate::storage::{PageMetadata, StorageQuery};

pub type TemplateSlug = String;

mod markdown;

pub struct Render {
    cache: Cache<StorageQuery, String>,
}

impl Render {
    pub fn init(cfg: &Config) -> Render {
        Render {
            cache: Cache::empty(1024), // TODO Get from config
        }
    }

    pub fn get_cache(&self, qry: &StorageQuery) -> Option<String> {
        // Get pre-rendered page from cache if any
        // TODO    Cache pre-rendered pages
        None
    }

    pub fn render_content(
        &self,
        body: String,
        md: &PageMetadata,
        page: &PageType,
        ctxt: &Context,
    ) -> String {
        // TODO    Render body from template using the engine
        "<html>TODO</html>".to_string()
    }
}
