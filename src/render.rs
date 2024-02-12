// TODO    Replace with tokio::sync::RwLock -> Make the whole Render struct async
use parking_lot::RwLock;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use syntect::{
    highlighting::ThemeSet,
    html::{css_for_theme_with_class_style, ClassStyle},
};
use tera::{try_get_value, Context, Tera};

use crate::{
    config::Configuration,
    errors::Errcode,
    post::{Post, PostMetadata, SerieMetadata},
};

use self::markdown::MarkdownRenderer;

pub mod markdown;

#[allow(unused_macros)]
macro_rules! minify_js {
    ($src:expr, $dst:expr) => {
        use minify_js::{minify, TopLevelMode};
        let code = tokio::fs::read(&$src).await?;
        let session = minify_js::Session::new();
        let mut out = Vec::new();
        minify(&session, TopLevelMode::Global, &code, &mut out).unwrap();
        tokio::fs::write($dst, out).await?;
    };
}

#[allow(dead_code)]
fn error_message(reason: String) -> String {
    format!(
        "<html>
        <link rel=\"stylesheet\" href=\"/style.css\"/>
        <body class=\"body-content-wrapper\">
        Unexpected error occured on this page: <br/>
        <pre><code>{reason}</code></pre>
        <a href=\"/\">Home</a>
        </body>
    </html>
    "
    )
}

fn get_template_from_cfg(cfg: &Arc<Configuration>, name: &'static str) -> Result<String, Errcode> {
    cfg.templates
        .get(name)
        .cloned()
        .ok_or(Errcode::TemplateTypeNotBound(name))
}

pub type RenderedPage = String;

pub struct Render {
    pub engine: RwLock<Tera>,
    pub base_context: Context,

    // Templates
    post_template: String,
    post_list_template: String,
    index_template: String,
    notification_template: String,

    #[allow(unused_variables)]
    pub config: Arc<Configuration>,
}

impl Render {
    pub async fn init(config: Arc<Configuration>) -> Result<Render, Errcode> {
        // TODO Once in a while, reload the template directory
        let mut tera =
            Tera::new(format!("{}/**/*.html", config.templates_dir.to_str().unwrap()).as_str())?;
        tera.register_filter("timestamp_convert", timestamp_to_date);
        tera.register_filter("markdown_render", markdown_render);

        Self::setup_css(&config).await?;
        Self::setup_scripts(&config).await?;

        let mut base_context = Context::new();

        #[cfg(feature = "add-endpoint")]
        crate::extensions::addendpoint::insert_additionnal_context(&config, &mut base_context)
            .await?;

        base_context.insert("site", &config.site_config);

        Ok(Render {
            base_context,
            engine: RwLock::new(tera),
            post_template: get_template_from_cfg(&config, "post")?,
            post_list_template: get_template_from_cfg(&config, "post_list")?,
            index_template: get_template_from_cfg(&config, "index")?,
            notification_template: get_template_from_cfg(&config, "notification")?,
            config,
        })
    }

    pub fn render_not_found(&self) -> RenderedPage {
        self.render_error("404 not found")
    }

    pub fn render_empty_post_list(&self, ptype: &'static str) -> RenderedPage {
        format!("No posts for this {}", ptype)
    }

    pub fn render_post_list(&self, ctxt: Context) -> Result<RenderedPage, Errcode> {
        self.render(&self.post_list_template, &ctxt)
    }

    pub fn render_list_allposts(
        &self,
        all_posts: Vec<PostMetadata>,
    ) -> Result<RenderedPage, Errcode> {
        if all_posts.is_empty() {
            return Ok(self.render_empty_post_list("allposts"));
        };
        let mut ctxt = self.base_context.clone();
        ctxt.insert("by", "all posts");
        ctxt.insert("all_posts", &all_posts);
        self.render(&self.post_list_template, &ctxt)
    }

    pub fn render_post(&self, post: Post, mut ctxt: Context) -> Result<RenderedPage, Errcode> {
        ctxt.insert("post_content", &post.content);
        ctxt.insert("nav", &post.post_nav);
        self.render(&self.post_template, &ctxt)
    }

    pub fn render_index(
        &self,
        recent: Vec<PostMetadata>,
        categories: Vec<String>,
        series: Vec<SerieMetadata>,
    ) -> Result<RenderedPage, Errcode> {
        let mut ctxt = self.base_context.clone();
        ctxt.insert("recent_posts", &recent);
        ctxt.insert("all_categories", &categories);
        ctxt.insert("all_series", &series);
        self.render(&self.index_template, &ctxt)
    }

    pub fn render_error<T: ToString>(&self, content: T) -> RenderedPage {
        let mut ctxt = self.base_context.clone();
        ctxt.insert("notif_h1", "Unexpected error occured");
        match self.render_notification("Error", content.to_string().as_str(), ctxt) {
            Ok(r) => r,
            Err(e) => {
                let mut errstr = format!("Error occured: {}<br/>", content.to_string());
                errstr += format!("Unable to display error page: {e:?}<br/>").as_str();
                errstr += "<a href=\"/\">Return to Index</a>";
                format!("<html>{errstr}</html>")
            }
        }
    }

    pub fn render_notification(
        &self,
        title: &str,
        content: &str,
        mut ctxt: Context,
    ) -> Result<RenderedPage, Errcode> {
        ctxt.insert("notif_title", title);
        ctxt.insert("notif_content", content);
        self.render(&self.notification_template, &ctxt)
    }

    pub fn render(&self, template: &str, ctxt: &Context) -> Result<RenderedPage, Errcode> {
        #[allow(unused_mut)]
        let mut rendered = self.engine.read().render(template, ctxt)?;

        #[cfg(feature = "html_minify")]
        let rendered = {
            let cfg = minify_html_onepass::Cfg {
                minify_js: true,
                minify_css: true,
            };
            match minify_html_onepass::in_place_str(&mut rendered, &cfg) {
                Ok(minified) => minified.to_string(),
                Err(e) => {
                    log::error!("Error while minifying: {:?}", e);
                    log::error!("{:?}", rendered.get((e.position - 50)..(e.position + 50)));
                    error_message(format!("Minifying HTML error: {:?}", e))
                }
            }
        };
        Ok(rendered)
    }

    pub async fn setup_css(config: &Configuration) -> Result<(), Errcode> {
        let grass_opts = config.get_grass_options();

        for (outpath, scss_list) in config.scss.iter() {
            let mut out_css = String::new();
            for scss_path in scss_list.iter() {
                let fpath = config.scss_dir.join(scss_path);
                if !fpath.exists() {
                    return Err(Errcode::PathDoesntExist("scss-file", fpath));
                }
                out_css += grass::from_path(fpath, &grass_opts)?.as_str();
            }

            #[cfg(feature = "css_minify")]
            let out_css = minify_css(outpath.to_string(), &out_css)?;

            tokio::fs::write(config.assets_dir.join(outpath), out_css).await?;
        }

        // Code.css
        let theme_set = ThemeSet::load_defaults();
        let theme = theme_set.themes.get(&config.code_theme).unwrap().clone();
        let code_css = css_for_theme_with_class_style(&theme, ClassStyle::Spaced)?;
        let fpath = config.assets_dir.join("code.css");
        #[cfg(feature = "css_minify")]
        let code_css = minify_css(format!("{:?}", &fpath), &code_css)?;
        tokio::fs::write(fpath, code_css).await?;

        Ok(())
    }

    pub async fn setup_scripts(config: &Configuration) -> Result<(), Errcode> {
        let mut all_files = tokio::fs::read_dir(&config.scripts_dir).await?;
        while let Some(file) = all_files.next_entry().await? {
            let src = file.path();
            if let Some(ext) = src.extension() {
                let fname = src.file_name().unwrap();
                #[cfg(feature = "js_minify")]
                if ext == "js" {
                    minify_js!(src, config.assets_dir.join(fname));
                }

                #[cfg(not(feature = "js_minify"))]
                if ext == "js" {
                    tokio::fs::copy(&src, config.assets_dir.join(fname)).await?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(feature = "css_minify")]
pub fn minify_css(name: String, css: &str) -> Result<RenderedPage, Errcode> {
    use lightningcss::stylesheet::{MinifyOptions, ParserOptions, PrinterOptions, StyleSheet};
    let parser_opts = ParserOptions {
        filename: name,
        ..Default::default()
    };
    let mut stylesheet = StyleSheet::parse(css, parser_opts)?;
    let minify_opts = MinifyOptions::default();
    stylesheet.minify(minify_opts)?;
    let printer_opts = PrinterOptions {
        minify: true,
        ..Default::default()
    };
    let res = stylesheet.to_css(printer_opts)?;
    Ok(res.code)
}

pub fn timestamp_to_date(val: &Value, _: &HashMap<String, Value>) -> Result<Value, tera::Error> {
    let s = try_get_value!("timestamp_to_date", "value", i64, val);
    let date = chrono::NaiveDateTime::from_timestamp_opt(s, 0).unwrap();
    let val = tera::to_value(date.format("%d/%m/%Y").to_string())?;
    Ok(val)
}

pub fn markdown_render(val: &Value, _: &HashMap<String, Value>) -> Result<Value, tera::Error> {
    let s = try_get_value!("markdown_render", "value", String, val)
        .trim()
        .to_string();
    let md = MarkdownRenderer::init();
    match md.render(s) {
        Err(e) => {
            log::error!("Error while rendering markdown in template:\n{e:?}");
            Err(tera::Error::msg(format!("Markdown render error: {e:?}")))
        }
        Ok((s, _)) => Ok(tera::to_value(s)?),
    }
}
