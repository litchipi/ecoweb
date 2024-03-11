use std::sync::Arc;

use parking_lot::RwLock;
use tera::Context;

use crate::cache::Cache;
use crate::config::Config;
use crate::page::PageMetadata;
use crate::errors::Errcode;
use crate::storage::{Storage, StorageQuery};

pub type TemplateSlug = String;

mod markdown;

pub struct Render {
    pub cache: Cache<StorageQuery, String>,
    templates_loaded: RwLock<Vec<String>>,
    storage: Arc<Storage>,
}

impl Render {
    pub fn init(storage: Arc<Storage>, cfg: &Config) -> Render {
        let qry = StorageQuery::template_base();
        // TODO    Create template engine here
        //    Load base templates
        //    For each base template added, register it in tracking vector
        let loaded = vec![];
        Render {
            storage,
            cache: Cache::empty(1024), // TODO Get from config
            templates_loaded: RwLock::new(loaded),
        }
    }

    pub fn has_template(&self, template: &String) -> bool {
        self.templates_loaded.read().contains(template)
    }

    pub async fn add_template(&self, template: String) -> Result<(), Errcode> {
        let qry = StorageQuery::template(template);
        let template = self.storage.query(qry).await.template()?;
        // TODO    Register into the engine
        self.templates_loaded.write().push(template);
        Ok(())
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
