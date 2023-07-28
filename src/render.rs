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
        })
    }

    pub fn render_empty_post_list(&self, ptype: &'static str) -> RenderedPage {
        format!("No posts for this {}", ptype)
    }

    pub fn render_post_list(&self, ctxt: Context) -> Result<RenderedPage, Errcode> {
        self.render("post_list.html", &ctxt)
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
        self.render("post_list.html", &ctxt)
    }

    pub fn render_post(
        &self,
        post: Post,
        nav: String,
        mut ctxt: Context,
    ) -> Result<RenderedPage, Errcode> {
        ctxt.insert("post_content", &post.content);
        ctxt.insert("nav", &nav);
        self.render("post.html", &ctxt)
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
        self.render("index.html", &ctxt)
    }

    pub fn render_rss_feed(&self, recent: Vec<PostMetadata>) -> Result<RenderedPage, Errcode> {
        let mut xml = "<rss version=\"2.0\"><channel>".to_string();
        xml += format!("<title>{}</title>", self.site_context.name).as_str();
        xml += format!("<link>{}</link>", self.site_context.base_url).as_str();
        xml += format!(
            "<description>{}</description>",
            self.site_context.description
        )
        .as_str();
        xml += format!(
            "<managingEditor>{}</managingEditor>",
            self.site_context.author_email
        )
        .as_str();
        xml += format!("<webMaster>{}</webMaster>", self.site_context.author_email).as_str();
        xml += format!("<copyright>{}</copyright>", self.site_context.copyrights).as_str();
        for post in recent {
            post.to_rss_item(&self.site_context, &mut xml);
        }
        xml += "</channel></rss>";
        Ok(xml)
    }

    pub fn render(&self, template: &'static str, ctxt: &Context) -> Result<RenderedPage, Errcode> {
        #[allow(unused_mut)]
        let mut rendered = self.engine.render(template, ctxt)?;

        std::fs::write("/tmp/page.html", &rendered)?;
        #[cfg(feature = "html_minify")]
        let rendered = {
            let cfg = minify_html_onepass::Cfg {
                minify_js: false,
                minify_css: false,
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

macro_rules! render_scss {
    ($config:expr, $($fpath:literal),* $(,)? => $outpath:literal) => {
        let grass_opts = $config.get_grass_options();
        let mut out_css = String::new();
        $(
            let fpath = $config.scss_dir.join($fpath);
            if !fpath.exists() {
                return Err(Errcode::PathDoesntExist("scss-file", fpath));
            }
            out_css += grass::from_path(fpath, &grass_opts)?.as_str();
        )*

        #[cfg(feature = "css_minify")]
        let out_css = minify_css($outpath.to_string(), &out_css)?;

        std::fs::write($config.assets_dir.join($outpath), out_css)?;
    };
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
    render_scss!(config,
        "base.scss",
        "banner.scss",
        "nav.scss",
        "post.scss",
        "specific.scss",
         => "style.css"
    );

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
