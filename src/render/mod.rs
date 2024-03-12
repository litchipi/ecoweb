use std::sync::Arc;

use parking_lot::RwLock;
use tera::{Context, Tera};

use crate::cache::Cache;
use crate::config::Config;
use crate::errors::Errcode;
use crate::page::PageMetadata;
use crate::storage::{Storage, StorageQuery};

pub type TemplateSlug = String;

mod markdown;

pub struct Render {
    pub cache: Cache<StorageQuery, String>,
    templates_loaded: RwLock<Vec<String>>,
    storage: Arc<Storage>,
    engine: Arc<RwLock<Tera>>,
}

impl Render {
    pub async fn init(storage: Arc<Storage>, cfg: &Config) -> Result<Render, Errcode> {
        let qry = StorageQuery::base_templates();

        let base_templates = storage.query(qry).await.base_templates()?;
        let loaded = base_templates.iter().map(|(n, _)| n.clone()).collect();

        let mut engine = Tera::default();
        engine.add_raw_templates(base_templates)?;
        Ok(Render {
            storage,
            cache: Cache::empty(1024), // TODO Get from config
            templates_loaded: RwLock::new(loaded),
            engine: Arc::new(RwLock::new(engine)),
        })
    }

    pub fn has_template(&self, template: &String) -> bool {
        self.templates_loaded.read().contains(template)
    }

    pub async fn add_template(&self, slug: String) -> Result<(), Errcode> {
        let qry = StorageQuery::template(slug.clone());
        let template = self.storage.query(qry).await.template()?;
        self.engine
            .write()
            .add_raw_template(slug.as_str(), template.as_str())?;
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
        Ok(format!(
            "<html><p>{body}</p><p>{md:?}</p><p>{ctxt:?}</p></html>"
        ))
    }

    pub async fn render_error(&self, err: Errcode) -> String {
        // TODO    Try to render error page
        //    If template doesn't exist, or fails to render, display a pure HTML message
        format!("{err:?}")
    }
}
