use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{config::Configuration, errors::Errcode};

#[derive(Clone, Serialize, Deserialize)]
pub struct SiteContext {
    pub name: String,
    pub base_url: String,
    pub og_image: Option<String>,
    pub author_name: String,
    pub author_email: String,
    pub description: String,
    welcome_message: String,
    pub copyrights: String,

    pub social: HashMap<String, String>,
    pub webring: WebringContext,

    #[serde(default)]
    pub humans_txt: String,

    blog_engine_src: Option<String>,
    blog_src: Option<String>,
}

impl SiteContext {
    #[allow(dead_code)]
    pub fn with_og_image(self, img_src: String) -> SiteContext {
        SiteContext {
            og_image: Some(img_src),
            ..self
        }
    }

    fn generate_humans_txt(&mut self) {
        self.humans_txt = String::new();
        self.humans_txt += "/* TEAM */\n";
        self.humans_txt += format!("Author: {}\n", self.author_name).as_str();
        for (sitename, social) in self.social.iter() {
            if sitename == "email" {
                let address = self.author_email.replace('@', " [at] ");
                self.humans_txt += format!("Email: {}\n", address).as_str();
            } else {
                let mut s = sitename.chars();
                let sitename_cap = match s.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + s.as_str(),
                };
                self.humans_txt += format!("{}: {}\n", sitename_cap, social).as_str();
            }
        }
        if let Some(ref blog_engine) = self.blog_engine_src {
            self.humans_txt += format!("\nSoftware sources: {}\n", blog_engine).as_str();
        }
        if let Some(ref blog_src) = self.blog_src {
            self.humans_txt += format!("Content sources: {}\n", blog_src).as_str();
        }
        self.humans_txt += "\nLanguage: English\n";
    }

    pub fn from_cfg(cfg: &Configuration) -> Result<SiteContext, Errcode> {
        let strdata = std::fs::read_to_string(&cfg.site_config_file)?;
        let mut ctxt: SiteContext = toml::from_str(&strdata)?;
        ctxt.generate_humans_txt();
        Ok(ctxt)
    }

    pub fn to_rss_feed(&self, xml: &mut String) {
        *xml += format!("<title>{}</title>", self.name).as_str();
        *xml += format!("<link>{}</link>", self.base_url).as_str();
        *xml += format!("<description>{}</description>", self.description).as_str();
        *xml += format!(
            "<managingEditor>{} ({})</managingEditor>",
            self.author_email, self.author_name,
        )
        .as_str();
        *xml += format!(
            "<webMaster>{} ({})</webMaster>",
            self.author_email, self.author_name
        )
        .as_str();
        *xml += format!("<copyright>{}</copyright>", self.copyrights).as_str();
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct WebringContext {
    name: String,
    next: String,
    previous: String,
}
