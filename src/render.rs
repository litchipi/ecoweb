use tera::Context;

use crate::{config::Config, storage::{Storage, PageMetadata, StorageQuery}, page::PageType};

pub type TemplateSlug = String;

pub struct Render {
    base_context: Context,
}

impl Render {
    pub fn init(ldr: &Storage, cfg: &Config) -> Render {
        // Initialise the context
        let base_context = Context::new();
        // Load additionnal context elements from storage, and insert it into base
        Render { 
            base_context,
        }
    }

    pub fn get_cache(&self, qry: &StorageQuery) -> Option<String> {
        // Get pre-rendered page from cache if any
        None
    }

    pub fn add_template(&self, ldr: &Storage, ptype: &PageType, md: &PageMetadata) {
        // Get template name from metadata
        // If not specified, template name is default one set in ptype
        // Get template from storage using name
        // Add to template to engine if doesn't exist
    }

    pub fn render_content(&self, body: String, md: &PageMetadata, page: &PageType, ctxt: &Context) -> String {
        "<html>TODO</html>".to_string()
    }

    pub fn build_context(&self, ldr: &Storage, md: &PageMetadata, page: &PageType) -> Context {
        let mut context = self.base_context.clone();
        for (name, data) in page.add_context.iter() {
            data.insert_context(ldr, name, &mut context);
        }

        for (name, data) in md.add_context.iter() {
            data.insert_context(ldr, name, &mut context);
        }
        context
    }
}

pub fn render_markdown(md: String, ctxt: &mut Context) -> String {
    // Render markdown to HTML
    // Add document structure to context
    "".to_string()
}
