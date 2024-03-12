use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::Hasher;

use serde::{Deserialize, Serialize};

use crate::render::TemplateSlug;
use crate::routes::ContentQueryMethod;
use crate::storage::{ContextQuery, StorageSlug};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PageMetadata {
    #[serde(default)]
    pub id: u64,

    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,

    #[serde(default)]
    pub add_context: HashMap<String, ContextQuery>,

    #[serde(default)]
    pub template: Option<String>,
}

impl PageMetadata {
    pub fn update_id(&mut self, page_name: String) {
        let mut s = DefaultHasher::new();
        s.write(format!("{:?}", self.metadata).as_bytes());
        self.id = s.finish();
    }
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
