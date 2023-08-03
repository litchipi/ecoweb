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
}

impl SiteContext {
    #[allow(dead_code)]
    pub fn with_og_image(self, img_src: String) -> SiteContext {
        SiteContext {
            og_image: Some(img_src),
            ..self
        }
    }

    pub fn from_cfg(cfg: &Configuration) -> Result<SiteContext, Errcode> {
        let strdata = std::fs::read_to_string(&cfg.site_config_file)?;
        let ctxt: SiteContext = toml::from_str(&strdata)?;
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
