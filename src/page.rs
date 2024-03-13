use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::Hasher;

use serde::{Deserialize, Serialize};

use crate::render::TemplateSlug;
use crate::routes::ContentQueryMethod;
use crate::storage::{ContextQuery, StorageSlug};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct PageMetadata {
    #[serde(default)]
    pub id: u64,

    #[serde(default)]
    pub hidden: bool,

    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,

    #[serde(default)]
    pub add_context: HashMap<String, ContextQuery>,

    #[serde(default)]
    pub template: Option<String>,
}

impl PageMetadata {
    pub fn get_metadata(&self, keys: &Vec<String>) -> Option<&serde_json::Value> {
        if keys.is_empty() {
            return None;
        }
        let mut keys_iter = keys.iter();
        let mut val = self.metadata.get(keys_iter.next().unwrap());
        for key in keys_iter {
            if let Some(data) = val {
                if data.is_object() {
                    val = data.as_object().unwrap().get(key);
                    continue;
                }
            } else {
                return None;
            }
        }
        val
    }

    pub fn update_id(&mut self, page_name: String) {
        let mut s = DefaultHasher::new();
        // TODO IMPORTANT FIXME   This is not constant, hash metadata manually instead
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
