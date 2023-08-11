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

use self::context::SiteContext;

pub mod context;
pub mod markdown;
pub mod nav;

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

type RenderedPage = String;

#[cfg(feature = "hot_reloading")]
mod tera_wrapper;

#[cfg(not(feature = "hot_reloading"))]
pub type TemplateEngine = Tera;

#[cfg(feature = "hot_reloading")]
pub type TemplateEngine = tera_wrapper::TeraWrapper;

pub struct Render {
    engine: TemplateEngine,
    pub base_context: Context,
    site_context: SiteContext,

    post_template: String,
    post_list_template: String,
    index_template: String,
    error_template: String,
}

impl Render {
    pub fn init(config: Arc<Configuration>) -> Result<Render, Errcode> {
        let mut tera =
            Tera::new(format!("{}/**/*.html", config.templates_dir.to_str().unwrap()).as_str())?;
        tera.register_filter("timestamp_convert", timestamp_to_date);

        setup_css(&config)?;
        setup_scripts(&config)?;

        #[cfg(feature = "hot_reloading")]
        let tera = tera_wrapper::TeraWrapper::new(config.clone());

        let mut base_context = Context::new();
        let site_context = context::SiteContext::from_cfg(config.as_ref())?;
        base_context.insert("site", &site_context);

        Ok(Render {
            site_context,
            base_context,
            engine: tera,
            post_template: config
                .templates
                .get("post")
                .cloned()
                .ok_or(Errcode::TemplateTypeNotBound("post"))?,
            post_list_template: config
                .templates
                .get("post_list")
                .cloned()
                .ok_or(Errcode::TemplateTypeNotBound("post_list"))?,
            index_template: config
                .templates
                .get("index")
                .cloned()
                .ok_or(Errcode::TemplateTypeNotBound("index"))?,
            error_template: config
                .templates
                .get("error")
                .cloned()
                .ok_or(Errcode::TemplateTypeNotBound("error"))?,
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

    pub fn render_post(
        &self,
        post: Post,
        nav: String,
        mut ctxt: Context,
    ) -> Result<RenderedPage, Errcode> {
        ctxt.insert("post_content", &post.content);
        ctxt.insert("nav", &nav);
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

    pub fn render_rss_feed(&self, recent: Vec<PostMetadata>) -> Result<RenderedPage, Errcode> {
        let mut xml = "<rss version=\"2.0\"><channel>".to_string();
        self.site_context.to_rss_feed(&mut xml);
        for post in recent {
            post.to_rss_item(&self.site_context, &mut xml);
        }
        xml += "</channel></rss>";
        Ok(xml)
    }

    pub fn render_error<T: ToString>(&self, content: T) -> RenderedPage {
        let mut ctxt = self.base_context.clone();
        ctxt.insert("error", &content.to_string());
        match self.engine.render(&self.error_template, &ctxt) {
            Ok(r) => r,
            Err(e) => {
                let mut errstr = format!("Error occured: {}<br/>", content.to_string());
                errstr += format!("Unable to display error page: {e:?}<br/>").as_str();
                errstr += "<a href=\"/\">Return to Index</a>";
                format!("<html>{errstr}</html>")
            }
        }
    }

    pub fn render(&self, template: &str, ctxt: &Context) -> Result<RenderedPage, Errcode> {
        #[allow(unused_mut)]
        let mut rendered = self.engine.render(template, ctxt)?;

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
                    println!("{:?}", rendered.get((e.position - 50)..(e.position + 50)));
                    error_message(format!("Minifying HTML error: {:?}", e))
                }
            }
        };
        Ok(rendered)
    }
}

#[cfg(feature = "css_minify")]
pub fn minify_css(name: String, css: &String) -> Result<RenderedPage, Errcode> {
    use lightningcss::stylesheet::{MinifyOptions, ParserOptions, PrinterOptions, StyleSheet};
    let mut parser_opts = ParserOptions::default();
    parser_opts.filename = name;
    let mut stylesheet = StyleSheet::parse(css, parser_opts)?;
    let minify_opts = MinifyOptions::default();
    stylesheet.minify(minify_opts)?;
    let mut printer_opts = PrinterOptions::default();
    printer_opts.minify = true;
    let res = stylesheet.to_css(printer_opts)?;
    Ok(res.code)
}

pub fn setup_css(config: &Arc<Configuration>) -> Result<(), Errcode> {
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

        std::fs::write(config.assets_dir.join(outpath), out_css)?;
    }

    // Code.css
    let theme_set = ThemeSet::load_defaults();
    let theme = theme_set.themes.get(&config.code_theme).unwrap().clone();
    let code_css = css_for_theme_with_class_style(&theme, ClassStyle::Spaced)?;
    let fpath = config.assets_dir.join("code.css");
    #[cfg(feature = "css_minify")]
    let code_css = minify_css(format!("{:?}", &fpath), &code_css)?;
    std::fs::write(fpath, code_css)?;

    Ok(())
}

#[allow(unused_macros)]
macro_rules! minify_js {
    ($fpath:expr) => {
        use minify_js::{minify, Session, TopLevelMode};
        let code = std::fs::read($fpath)?;
        let session = Session::new();
        let mut out = Vec::new();
        minify(&session, TopLevelMode::Global, &code, &mut out).unwrap();
        std::fs::write($fpath, out)?;
    };
}

pub fn setup_scripts(config: &Arc<Configuration>) -> Result<(), Errcode> {
    {
        let script = &"post.js";
        std::fs::copy(
            config.scripts_dir.join(script),
            config.assets_dir.join(script),
        )?;

        #[cfg(feature = "js_minify")]
        minify_js!(config.assets_dir.join(script));
    }

    Ok(())
}

pub fn timestamp_to_date(val: &Value, _: &HashMap<String, Value>) -> Result<Value, tera::Error> {
    let s = try_get_value!("timestamp_to_date", "value", i64, val);
    let date = chrono::NaiveDateTime::from_timestamp_opt(s, 0).unwrap();
    let val = tera::to_value(date.format("%d/%m/%Y").to_string())?;
    Ok(val)
}
