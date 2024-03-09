use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::render::TemplateSlug;
use crate::routes::UrlBuildMethod;
use crate::storage::{ContextQuery, StorageSlug};

#[derive(Debug, Serialize, Deserialize)]
pub struct PageMetadata {
    pub add_context: HashMap<String, ContextQuery>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PageType {
    pub route: String,
    pub lang_detect: bool,
    pub add_context: HashMap<String, ContextQuery>,
    pub default_template: TemplateSlug,
    url_build_method: UrlBuildMethod,
    pub storage: StorageSlug,
}

impl PageType {
    pub fn test() -> PageType {
        PageType {
            route: "/toto".to_string(),
            lang_detect: false,
            add_context: HashMap::new(),
            default_template: "index.html".to_string(),
            url_build_method: UrlBuildMethod::ContentId,
            storage: "dev".to_string(),
        }
    }
}
