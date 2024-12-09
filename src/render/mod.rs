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
    notification_template: String,
}

impl Render {
    pub async fn init(storage: Arc<Storage>, cfg: &Config) -> Result<Render, Errcode> {
        let engine = Self::init_engine(&storage).await?;
        Ok(Render {
            storage,
            engine: Arc::new(RwLock::new(engine)),
            markdown_render: MarkdownRenderer::init(),
            notification_template: cfg.notification_template.clone(),
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
        template: &str,
        body: String,
        mut ctxt: Context,
    ) -> Result<String, Errcode> {
        #[cfg(feature = "hot-reloading")]
        {
            *self.engine.write() = Self::init_engine(&self.storage).await?;
        }

        self.markdown_render.render_to_ctxt(body, &mut ctxt)?;
        let result = self.engine.read().render(template, &ctxt)?;
        Ok(result)
    }

    pub async fn render_error(&self, err: &Errcode, ctxt: Context) -> String {
        match self
            .render_notification("Error".to_string(), format!("{err:?}"), ctxt)
            .await
        {
            Ok(body) => body,
            Err(e) => {
                format!(
                    "
                    <html>
                        <body>
                            <h1>Error while displaying the error page</h1>
                            <p>Render error on error page:</p>
                            <pre><code>{e:?}</code></pre>
                            <h2>Occured while treating the following error</h2>
                            <pre><code>{err:?}</code></pre>
                        </body>
                    </html>"
                )
            }
        }
    }

    pub async fn render_notification(
        &self,
        title: String,
        msg: String,
        mut ctxt: Context,
    ) -> Result<String, Errcode> {
        #[cfg(feature = "hot-reloading")]
        {
            *self.engine.write() = Self::init_engine(&self.storage).await?;
        }

        ctxt.insert("notif_title", &title);
        ctxt.insert("notif_content", &msg);

        let result = self
            .engine
            .read()
            .render(&self.notification_template, &ctxt)?;

        Ok(result)
    }

    #[cfg(feature = "html_minify")]
    pub fn minify(&self, page: &String) -> Result<String, Errcode> {
        let data = minify_html::minify(page.as_bytes(), &minify_html::Cfg::default());
        String::from_utf8(data).or(Err(Errcode::MinificationFailed))
    }
}

pub fn timestamp_to_date(
    val: &tera::Value,
    _: &HashMap<String, tera::Value>,
) -> Result<tera::Value, tera::Error> {
    let s = try_get_value!("timestamp_to_date", "value", i64, val);
    let date = chrono::DateTime::from_timestamp(s, 0).unwrap();
    let val = tera::to_value(date.format("%d/%m/%Y").to_string())?;
    Ok(val)
}
