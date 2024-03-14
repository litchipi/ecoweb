use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

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
        s.write_u8(if self.hidden { 1 } else { 0 });
        let mut keys: Vec<&String> = self.metadata.keys().collect();
        keys.sort();
        for key in keys {
            s.write(key.as_bytes());
            hash_json(&mut s, self.metadata.get(key).unwrap());
        }
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

pub fn hash_json(s: &mut DefaultHasher, val: &serde_json::Value) {
    match val {
        tera::Value::Null => s.write_u8(0),
        tera::Value::Bool(b) => s.write_u8(if *b { 1 } else { 0 }),
        tera::Value::Number(n) => n.hash(s),
        tera::Value::String(t) => t.hash(s),
        tera::Value::Array(arr) => arr.iter().for_each(|v| hash_json(s, v)),
        tera::Value::Object(map) => {
            let keys: Vec<&String> = map.keys().collect();
            for k in keys {
                s.write(k.as_bytes());
                hash_json(s, map.get(k).unwrap());
            }
        },
    }
}
