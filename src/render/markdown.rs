use std::collections::HashMap;

use tera::Context;
use syntect::html::{ClassStyle, ClassedHTMLGenerator};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;
use mdtrans::{MarkdownTransformer, transform_markdown_string};

#[derive(Clone)]
pub struct MarkdownRenderer {}

impl MarkdownRenderer {
    pub fn init() -> MarkdownRenderer {
        MarkdownRenderer {}
    }

    pub fn render(&self, content: String, ctxt: &mut Context) -> Result<(), mdtrans::Errcode> {
        let mut transformer = MarkdownToHtml::init();
        let mut body = transform_markdown_string(content, &mut transformer)?;
        if transformer.current_section > 0 {
            body += "</section>";
        }

        let nav = self.render_nav(transformer.sections);
        ctxt.insert("page-content", &body);
        ctxt.insert("page-nav", &nav);
        Ok(())
    }

    fn render_nav(&self, sections: Vec<(usize, String, String)>) -> String {
        let mut nav = String::new();
        let mut last_level = 0;
        for (level, title, slug) in sections {
            let link = format!("<a href=\"#{slug}\" class=\"h{level}\">{title}</a>");
            if level == last_level {
                nav += "<li>";
                nav += link.as_str();
                nav += "</li>";
            }
            if level > last_level {
                if last_level > 0 {
                    nav += "<li>";
                }
                while level > last_level {
                    nav += "<ul><li>";
                    last_level += 1;
                }
                nav += link.as_str();
                nav += "</li>";
            }
            if level < last_level {
                while level < last_level {
                    nav += "</ul></li>";
                    last_level -= 1;
                }
                nav += "<li>";
                nav += link.as_str();
                nav += "</li>";
            }
            last_level = level;
            nav += "\n";
        }
        while last_level > 1 {
            nav += "</ul></li>";
            last_level -= 1;
        }
        nav += "</ul>";
        nav
    }
}

struct MarkdownToHtml {
    refs: HashMap<String, String>,
    syntax_set: SyntaxSet,

    sections: Vec<(usize, String, String)>,
    current_section: usize,
}

impl MarkdownToHtml {
    fn init() -> MarkdownToHtml {
        MarkdownToHtml {
            refs: HashMap::new(),
            syntax_set: SyntaxSet::load_defaults_newlines(),

            sections: vec![],
            current_section: 0,
        }
    }
    fn sanitize_html(&self, text: String) -> String {
        text.replace('<', "&lt;").replace('>', "&gt;")
    }

    fn slugify_header(&self, text: &str) -> String {
        format!(
            "{}-{}",
            self.sections.len(),
            text.to_lowercase()
                .replace(' ', "-")
                .chars()
                .filter(|c| *c == '-' || c.is_alphanumeric())
                .collect::<String>()
        )
    }
}

impl MarkdownTransformer for MarkdownToHtml {
    fn transform_horizontal_separator(&mut self) -> String {
        "<hr>".to_string()
    }

    fn transform_text(&mut self, text: String) -> String {
        self.sanitize_html(text)
    }

    fn transform_quote(&mut self, text: String) -> String {
        format!("<blockquote>{text}</blockquote>")
    }

    fn transform_image(
        &mut self,
        alt: String,
        url: String,
        add_tags: std::collections::HashMap<String, String>,
    ) -> String {
        let mut metadata = " ".to_string();
        metadata += add_tags
            .into_iter()
            .map(|(key, val)| {
                let val = val.trim_start_matches('\"').trim_end_matches('\"');
                format!(" {key}=\"{val}\"")
            })
            .collect::<Vec<String>>()
            .join(" ")
            .as_str();
        format!("<img src=\"{url}\" alt=\"{alt}\"{metadata}>")
    }

    fn transform_bold(&mut self, text: String) -> String {
        format!("<strong>{text}</strong>")
    }

    fn transform_italic(&mut self, text: String) -> String {
        format!("<em>{text}</em>")
    }

    fn transform_link(&mut self, text: String, url: String) -> String {
        format!("<a href=\"{url}\">{text}</a>")
    }

    fn transform_header(&mut self, level: usize, text: String) -> String {
        let mut buffer = "".to_string();
        if self.current_section > 0 {
            buffer += "</section>";
        }
        let (sec_level, _, slug) = self.sections.get(self.current_section).unwrap();
        assert_eq!(sec_level, &level);
        buffer += format!("<section id=\"{slug}\">").as_str();
        buffer += format!("<h{level}>{text}</h{level}>").as_str();
        self.current_section += 1;
        buffer
    }

    fn peek_header(&mut self, level: usize, text: String) {
        let slug = self.slugify_header(&text);
        self.sections.push((level, text, slug));
    }

    fn transform_inline_code(&mut self, text: String) -> String {
        format!("<code>{}</code>", self.sanitize_html(text))
    }

    fn transform_codeblock(&mut self, lang: Option<String>, text: String) -> String {
        let code = {
            if let Some(l) = lang {
                if let Some(syntax) = self.syntax_set.find_syntax_by_token(&l) {
                    let mut html_generator = ClassedHTMLGenerator::new_with_class_style(
                        syntax,
                        &self.syntax_set,
                        ClassStyle::Spaced,
                    );
                    let mut is_err = false;
                    for line in LinesWithEndings::from(&text) {
                        if let Err(e) =
                            html_generator.parse_html_for_line_which_includes_newline(line)
                        {
                            log::error!("Got error when generating html from code line: {e:?}");
                            is_err = true;
                            break;
                        }
                    }
                    if is_err {
                        self.sanitize_html(text)
                    } else {
                        html_generator.finalize()
                    }
                } else {
                    self.sanitize_html(text)
                }
            } else {
                self.sanitize_html(text)
            }
        };
        format!("<pre><code>{code}</code></pre>")
    }

    fn peek_refurl(&mut self, slug: String, url: String) {
        self.refs.insert(slug, url);
    }

    fn transform_reflink(&mut self, text: String, slug: String) -> String {
        let url = self.refs.get(&slug);
        assert!(url.is_some(), "Link reference {slug} not found");
        self.transform_link(text, url.unwrap().clone())
    }

    fn transform_refurl(&mut self, _slug: String, _url: String) -> String {
        "".to_string()
    }

    fn transform_list(&mut self, elements: Vec<String>) -> String {
        let mut buffer = "<ul>\n".to_string();
        buffer += elements.join("\n").as_str();
        buffer += "\n</ul>";
        buffer
    }

    fn transform_list_element(&mut self, element: String) -> String {
        format!("<li>{}</li>", element)
    }

    fn transform_paragraph(&mut self, text: String) -> String {
        format!("<p>{text}</p>")
    }

    fn transform_vertical_space(&mut self) -> String {
        "<br/>".to_string()
    }

    fn transform_comment(&mut self, text: String) -> String {
        format!("<!-- {text} -->")
    }
}
