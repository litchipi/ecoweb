use crate::{config::Config, storage::{Storage, PageMetadata, StorageQuery}, page::PageType};

pub type TemplateSlug = String;

pub struct Render {
    
}

impl Render {
    pub fn init(ldr: &Storage, cfg: &Config) -> Render {
        // Initialise the context
        // Load additionnal context elements from storage, and insert it into base
        Render { }
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

    pub fn render_content(&self, body: String, md: &PageMetadata, page: &PageType) -> String {
        "<html>TODO</html>".to_string()
    }

    pub fn build_context(&self, ldr: &Storage, md: &PageMetadata, page: &PageType) -> Context {
        let mut context = self.base_context.clone();
        for add in page.add_context.iter() {
            add.insert_context(ldr, &mut context);
        }

        for add in md.add_context.iter() {
            add.insert_context(ldr, &mut context);
        }
        context
    }
}

pub fn render_markdown(md: String, ctxt: &mut Context) -> String {
    // Render markdown to HTML
    // Add document structure to context
    "".to_string()
}
