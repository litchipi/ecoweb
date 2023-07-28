use std::sync::Arc;
use tera::{Context, Tera};

use super::{context::SiteContext, setup_css, setup_scripts, timestamp_to_date};
use crate::{config::Configuration, errors::Errcode};

pub struct TeraWrapper {
    config: Arc<Configuration>,
}

impl TeraWrapper {
    pub fn new(config: Arc<Configuration>) -> TeraWrapper {
        TeraWrapper { config }
    }

    pub fn render(&self, name: &str, ctxt: &Context) -> Result<String, Errcode> {
        log::debug!("Hot reloading");
        let mut ctxt = ctxt.clone();

        // Override site context
        let site_context = SiteContext::from_cfg(self.config.as_ref())?;
        ctxt.insert("site", &site_context);

        // Write changes to SCSS
        setup_css(&self.config)?;
        setup_scripts(&self.config)?;

        // Reload templates
        let mut tera = Tera::new(
            format!("{}/**/*.html", self.config.templates_dir.to_str().unwrap()).as_str(),
        )?;
        tera.register_filter("timestamp_convert", timestamp_to_date);

        let rendered = tera.render(name, &ctxt)?;
        Ok(rendered)
    }
}
