use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;
use tera::{try_get_value, Context, Tera};

use crate::cache::Cache;
use crate::config::Config;
use crate::errors::Errcode;
use crate::page::PageMetadata;
use crate::storage::{Storage, StorageQuery};

use self::markdown::MarkdownRenderer;

pub type TemplateSlug = String;

mod markdown;

pub struct Render {
    pub cache: Cache<StorageQuery, String>,
    templates_loaded: RwLock<Vec<String>>,
    storage: Arc<Storage>,
    engine: Arc<RwLock<Tera>>,
    markdown_render: MarkdownRenderer,
}

impl Render {
    pub async fn init(storage: Arc<Storage>, cfg: &Config) -> Result<Render, Errcode> {
        // TODO    Register ALL templates from database directly here
        //     Remove the per-request template registering
        let qry = StorageQuery::base_templates();

        let base_templates = storage.query(qry).await.base_templates()?;
        let loaded = base_templates.iter().map(|(n, _)| n.clone()).collect();

        let mut engine = Tera::default();
        engine.register_filter("timestamp_convert", timestamp_to_date);
        engine.register_filter("markdown_render", markdown::markdown_render);
        engine.add_raw_templates(base_templates)?;
        Ok(Render {
            storage,
            cache: Cache::empty(1024), // TODO Get from config
            templates_loaded: RwLock::new(loaded),
            engine: Arc::new(RwLock::new(engine)),
            markdown_render: MarkdownRenderer::init(),
        })
    }

    pub async fn add_template(&self, slug: &String) -> Result<(), Errcode> {
        if self.templates_loaded.read().contains(slug) {
            return Ok(());
        }

        let qry = StorageQuery::template(slug.clone());
        let template = self.storage.query(qry).await.template()?;
        self.engine
            .write()
            .add_raw_template(slug, template.as_str())?;
        self.templates_loaded.write().push(template);
        Ok(())
    }

    pub async fn render_content(
        &self,
        template: &String,
        body: String,
        md: &PageMetadata,
        mut ctxt: Context,
    ) -> Result<String, Errcode> {
        self.add_template(template).await?;
        self.markdown_render.render_to_ctxt(body, &mut ctxt)?;
        let result = self.engine.read().render(template, &ctxt)?;
        Ok(result)
    }

    pub async fn render_error(&self, err: &Errcode) -> String {
        // TODO    Try to render error page
        //    If template doesn't exist, or fails to render, display a pure HTML message
        format!("<html>Error: {err:?}</html>")
    }
}

pub fn timestamp_to_date(
    val: &tera::Value,
    _: &HashMap<String, tera::Value>
) -> Result<tera::Value, tera::Error> {
    let s = try_get_value!("timestamp_to_date", "value", i64, val);
    let date = chrono::NaiveDateTime::from_timestamp_opt(s, 0).unwrap();
    let val = tera::to_value(date.format("%d/%m/%Y").to_string())?;
    Ok(val)
}
