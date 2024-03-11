use tera::Context;

use crate::cache::Cache;
use crate::config::Config;
use crate::page::{PageMetadata, PageType};
use crate::errors::Errcode;
use crate::storage::StorageQuery;

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

    pub async fn render_content(
        &self,
        body: String,
        md: &PageMetadata,
        page: &PageType,
        ctxt: &Context,
    ) -> String {
        // TODO    Render body from template using the engine
        "<html>TODO</html>".to_string()
    }

    pub async fn render_error(&self, err: Errcode) -> String {
        // TODO    Try to render error page
        //    If template doesn't exist, or fails to render, display a pure HTML message
        format!("{err:?}")
    }
}
