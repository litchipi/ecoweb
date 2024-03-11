use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::render::TemplateSlug;
use crate::routes::ContentQueryMethod;
use crate::storage::{ContextQuery, StorageSlug};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PageMetadata {
    #[serde(default)]
    pub add_context: HashMap<String, ContextQuery>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PageType {
    pub route: String,
    pub lang_detect: bool,
    
    #[serde(default)]
    pub add_context: HashMap<String, ContextQuery>,
    pub default_template: TemplateSlug,

    #[serde(default)]
    pub content_query: ContentQueryMethod,

    #[serde(default)]
    pub storage: StorageSlug,
}
