use tera::Context;

use crate::config::Config;
use crate::page::PageType;
use crate::errors::Errcode;
use crate::storage::{PageMetadata, Storage, StorageQuery};
use crate::cache::Cache;

pub type TemplateSlug = String;

mod markdown;

pub struct Render {
    base_context: Context,
    cache: Cache<StorageQuery, String>,
}

impl Render {
    pub fn init(cfg: &Config) -> Render {
        // TODO    Create base context
        let base_context = Context::new();
        Render { 
            base_context,
            cache: Cache::empty(),
        }
    }

    pub fn get_cache(&self, qry: &StorageQuery) -> Option<String> {
        // Get pre-rendered page from cache if any
        // TODO    Cache pre-rendered pages
        None
    }

    pub fn add_template(&self, ldr: &Storage, ptype: &PageType, md: &PageMetadata) {
        // TODO    Check if template is already loaded or not
        // TODO    Load template from storage if not loaded
        // TODO    Add template to engine
    }

    pub fn render_content(&self, body: String, md: &PageMetadata, page: &PageType, ctxt: &Context) -> String {
        // TODO    Render body from template using the engine
        "<html>TODO</html>".to_string()
    }

    pub fn build_context(&self, ldr: &Storage, md: &PageMetadata, page: &PageType) -> Result<Context, Errcode> {
        let mut context = self.base_context.clone();
        for (name, data) in page.add_context.iter() {
            data.insert_context(ldr, name, &mut context)?;
        }
        for (name, data) in md.add_context.iter() {
            data.insert_context(ldr, name, &mut context)?;
        }
        Ok(context)
    }
}
