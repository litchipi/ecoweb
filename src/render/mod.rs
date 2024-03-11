use tera::Context;

use crate::cache::Cache;
use crate::config::Config;
use crate::page::PageMetadata;
use crate::errors::Errcode;
use crate::storage::StorageQuery;

pub type TemplateSlug = String;

mod markdown;

pub struct Render {
    pub cache: Cache<StorageQuery, String>,
}

impl Render {
    pub fn init(cfg: &Config) -> Render {
        Render {
            cache: Cache::empty(1024), // TODO Get from config
        }
    }

    pub async fn render_content(
        &self,
        body: String,
        md: &PageMetadata,
        ctxt: &Context,
    ) -> Result<String, Errcode> {
        // TODO    Render body from template using the engine
        Ok(format!("<html><p>{body}</p><p>{md:?}</p><p>{ctxt:?}</p></html>"))
    }

    pub async fn render_error(&self, err: Errcode) -> String {
        // TODO    Try to render error page
        //    If template doesn't exist, or fails to render, display a pure HTML message
        format!("{err:?}")
    }
}
