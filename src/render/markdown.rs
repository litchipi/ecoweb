use std::collections::HashMap;
use std::io::Write;

use comrak::adapters::SyntaxHighlighterAdapter;
use comrak::html::escape;
use comrak::{markdown_to_html_with_plugins, ComrakOptions, ComrakPlugins};
use regex::Regex;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::html::{ClassStyle, ClassedHTMLGenerator};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use crate::config::Configuration;
use crate::post::PostMetadata;

const SECTIONIZE_REGEX: &str =
    "<h(\\d)>\\s?<a href=\"#(.*?)\" .*? class=\"anchor\".*?>[\\s\\S]*?</h(\\d)>";

const IMAGES_INDEXER_REGEX: &str = "<img(.*?)/>";

#[derive(Clone)]
pub struct MarkdownRenderer {
    pub theme: Theme,
    syntax_set: SyntaxSet,
    comrak_opts: ComrakOptions,
    sectionize_re: Regex,
    image_indexer_re: Regex,
}

impl MarkdownRenderer {
    pub fn init(cfg: &Configuration) -> MarkdownRenderer {
        let theme_set = ThemeSet::load_defaults();
        let syntax_set = SyntaxSet::load_defaults_newlines();

        let mut comrak_opts = ComrakOptions::default();
        comrak_opts.extension.header_ids = Some("".to_string());

        let sectionize_re = Regex::new(SECTIONIZE_REGEX).unwrap();
        let image_indexer_re = Regex::new(IMAGES_INDEXER_REGEX).unwrap();

        MarkdownRenderer {
            theme: theme_set.themes.get(&cfg.code_theme).unwrap().clone(),
            comrak_opts,
            syntax_set,
            sectionize_re,
            image_indexer_re,
        }
    }

    pub fn highlight(&self, code: &str, lang: &str) -> Result<String, syntect::Error> {
        let Some(syntax) = self.syntax_set.find_syntax_by_token(lang) else {
            let mut res = vec![];
            escape(&mut res, code.as_bytes())?;
            return Ok(String::from_utf8(res).unwrap());
        };
        let mut html_generator = ClassedHTMLGenerator::new_with_class_style(
            syntax,
            &self.syntax_set,
            ClassStyle::Spaced,
        );
        for line in LinesWithEndings::from(code) {
            html_generator.parse_html_for_line_which_includes_newline(line)?;
        }
        let output_html = html_generator.finalize();
        Ok(output_html)
    }

    pub fn render(&self, content: String, metadata: &PostMetadata) -> String {
        let mut plugins = ComrakPlugins::default();
        plugins.render.codefence_syntax_highlighter = Some(self);
        let html_base = markdown_to_html_with_plugins(&content, &self.comrak_opts, &plugins);

        let mut html_sec = String::new();
        let mut last = None;
        let mut nb_open = 0;
        for cap in self.sectionize_re.captures_iter(&html_base) {
            let lvl_cap = cap.get(1).unwrap();
            let end = lvl_cap.start() - 2;
            let lvl: usize = lvl_cap.as_str().parse().unwrap();
            let id = cap[2].to_string();

            if let Some((mut last_lvl, start)) = last {
                html_sec += &html_base[start..end];
                while lvl <= last_lvl {
                    html_sec += "</section>";
                    last_lvl -= 1;
                    nb_open -= 1;
                    assert!(nb_open >= 0);
                }
            } else {
                html_sec += &html_base[..(lvl_cap.start() - 2)];
            }
            let start_next = cap.get(3).unwrap().end() + 1;
            html_sec += format!("<section id={}>", id).as_str();
            nb_open += 1;
            html_sec += &cap[0];
            last = Some((lvl, start_next));
        }
        if let Some((_, start)) = last {
            html_sec += &html_base[start..];
            while nb_open > 0 {
                html_sec += "</section>";
                nb_open -= 1;
            }
        }

        let mut html_imgind = String::new();
        let mut tail = 0;
        // TODO    Find a way to get the filename of the source as a slug to be used in key
        //    Instead of the image index
        for (n, cap) in self.image_indexer_re.captures_iter(&html_sec).enumerate() {
            log::debug!("Image add attributes: {:?}", metadata.images_add_attribute);
            let cap = cap.get(1).unwrap();
            let start = cap.start() - 4;
            html_imgind += &html_sec[tail..start];
            html_imgind += format!(
                "<img id=\"{n}\" {} {}",
                cap.as_str(),
                if let Some(s) = metadata.images_add_attribute.get(&format!("{n}")) {
                    s
                } else {
                    ""
                }
            )
            .as_str();
            tail = cap.end() + 1;
        }
        html_imgind += &html_sec[tail..];
        html_imgind
    }
}

impl SyntaxHighlighterAdapter for MarkdownRenderer {
    fn write_highlighted(
        &self,
        output: &mut dyn Write,
        lang: Option<&str>,
        code: &str,
    ) -> std::io::Result<()> {
        if let Some(l) = lang {
            write!(output, "<span class=\"lang-{}\">", l)?;
            write!(output, "{}", self.highlight(code, l).unwrap())?;
        } else {
            write!(output, "<span class=\"nolang\">")?;
            escape(output, code.as_bytes())?;
        }
        write!(output, "</span>")
    }

    fn write_pre_tag(
        &self,
        output: &mut dyn Write,
        _attributes: HashMap<String, String>,
    ) -> std::io::Result<()> {
        output.write_all(b"<pre>")
    }

    fn write_code_tag(
        &self,
        output: &mut dyn Write,
        attributes: HashMap<String, String>,
    ) -> std::io::Result<()> {
        if let Some(l) = attributes.get("class") {
            output.write_all(format!("<code class=\"{}\">", l).as_bytes())
        } else {
            output.write_all(b"<code class=\"nolang\">")
        }
    }
}
