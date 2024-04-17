use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;
use tera::{try_get_value, Context, Tera};

use crate::config::Config;
use crate::errors::Errcode;
use crate::storage::{Storage, StorageQuery};

use self::markdown::MarkdownRenderer;

pub type TemplateSlug = String;

mod markdown;

pub struct Render {
    storage: Arc<Storage>,
    engine: Arc<RwLock<Tera>>,
    markdown_render: MarkdownRenderer,
}

impl Render {
    pub async fn init(storage: Arc<Storage>, cfg: &Config) -> Result<Render, Errcode> {
        let engine = Self::init_engine(&storage).await?;
        Ok(Render {
            storage,
            engine: Arc::new(RwLock::new(engine)),
            markdown_render: MarkdownRenderer::init(),
        })
    }

    pub async fn init_engine(storage: &Arc<Storage>) -> Result<Tera, Errcode> {
        let qry = StorageQuery::templates();
        let base_templates = storage.query(qry).await.base_templates()?;
        let mut engine = Tera::default();
        engine.register_filter("timestamp_convert", timestamp_to_date);
        engine.register_filter("markdown_render", markdown::markdown_render);
        engine.add_raw_templates(base_templates)?;
        Ok(engine)
    }

    pub async fn render_content(
        &self,
        template: &String,
        body: String,
        mut ctxt: Context,
    ) -> Result<String, Errcode> {
        #[cfg(feature = "hot-reloading")]
        {
            *self.engine.write() = Self::init_engine(&self.storage).await?;
        }

        self.markdown_render.render_to_ctxt(body, &mut ctxt)?;
        let result = self.engine.read().render(template, &ctxt)?;

        #[cfg(feature = "html_minify")]
        let result = minify_html(result);
        
        Ok(result)
    }

    pub async fn render_error(&self, err: &Errcode) -> String {
        log::warn!("Returning error to client:\n{err:?}");
        // TODO    Try to render error page
        //    If template doesn't exist, or fails to render, display a pure HTML message
        format!("<html><body><h1>Error:</h1><pre><code>{err:?}</code></pre></body></html>")
    }
}

pub fn timestamp_to_date(
    val: &tera::Value,
    _: &HashMap<String, tera::Value>
) -> Result<tera::Value, tera::Error> {
    let s = try_get_value!("timestamp_to_date", "value", i64, val);
    let date = chrono::DateTime::from_timestamp(s, 0).unwrap();
    let val = tera::to_value(date.format("%d/%m/%Y").to_string())?;
    Ok(val)
}

// TODO    Minify HTML
pub fn minify_html(html: String) -> String {
    html
}
