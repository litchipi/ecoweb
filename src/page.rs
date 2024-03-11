use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::render::TemplateSlug;
use crate::routes::UrlBuildMethod;
use crate::storage::{ContextQuery, StorageSlug};

#[derive(Debug, Serialize, Deserialize)]
pub struct PageMetadata {
    pub add_context: HashMap<String, ContextQuery>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PageType {
    pub route: String,
    pub lang_detect: bool,
    
    #[serde(default)]
    pub add_context: HashMap<String, ContextQuery>,
    pub default_template: TemplateSlug,
    pub url_build_method: UrlBuildMethod,

    #[serde(default)]
    pub storage: StorageSlug,
}

impl PageType {
    pub fn test() -> PageType {
        let mut add_context = HashMap::new();
        add_context.insert(
            "page_type_test".to_string(),
            ContextQuery::Plain(serde_json::Value::Number(23.into())),
        );
        PageType {
            route: "/toto".to_string(),
            lang_detect: false,
            add_context,
            default_template: "index.html".to_string(),
            url_build_method: UrlBuildMethod::ContentId,
            storage: "dev".to_string(),
        }
    }
}
