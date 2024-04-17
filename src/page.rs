use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};
use tera::Value;

use crate::render::TemplateSlug;
use crate::routes::ContentQueryMethod;
use crate::storage::{ContextQuery, StorageSlug};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct PageMetadata {
    // FIXME    u64 deserialization with toml
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
    pub fn compare_md(&self, keys: &Vec<String>, other: &Self) -> std::cmp::Ordering {
        let data = self.get_metadata(keys);
        let other = other.get_metadata(keys);
        let ord = compare_tera_values(data, other);
        // By default, get from greater to lower
        ord.reverse()
    }

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

    #[serde(default)]
    pub lang_detect: bool,

    #[serde(default)]
    pub add_context: HashMap<String, ContextQuery>,
    pub default_template: TemplateSlug,

    #[serde(default)]
    pub content_query: ContentQueryMethod,

    #[serde(default)]
    pub storage: StorageSlug,

    #[serde(default)]
    pub add_headers: HashMap<String, String>,
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
        }
    }
}

// Implementation to compare values of metadata
pub fn compare_tera_values(a: Option<&Value>, b: Option<&Value>) -> std::cmp::Ordering {
    match (a, b) {
        (None, None) => std::cmp::Ordering::Equal,
        (Some(_), None) => std::cmp::Ordering::Greater,
        (None, Some(_)) => std::cmp::Ordering::Less,
        (Some(a), Some(b)) => match a {
            tera::Value::Null if b.is_null() => std::cmp::Ordering::Equal,
            tera::Value::Bool(d) if b.is_boolean() => d.cmp(&b.as_bool().unwrap()),
            tera::Value::Number(n) if b.is_number() => {
                if n.is_i64() && b.is_i64() {
                    n.as_i64().unwrap().cmp(&b.as_i64().unwrap())
                } else if n.is_u64() && b.is_u64() {
                    n.as_u64().unwrap().cmp(&b.as_u64().unwrap())
                } else if n.is_f64() && b.is_f64() {
                    n.as_f64()
                        .unwrap()
                        .partial_cmp(&b.as_f64().unwrap())
                        .unwrap_or(std::cmp::Ordering::Equal)
                } else {
                    unreachable!()
                }
            }
            tera::Value::String(s) if b.is_string() => s.cmp(&b.as_str().unwrap().to_string()),
            tera::Value::Array(v) if b.is_array() => {
                for el_a in v {
                    for el_b in b.as_array().unwrap() {
                        let ord = compare_tera_values(Some(el_a), Some(el_b));
                        if ord.is_ne() {
                            return ord;
                        }
                    }
                }
                std::cmp::Ordering::Equal
            }
            tera::Value::Object(obj_a) if b.is_object() => {
                let obj_b = b.as_object().unwrap();
                let mut keys_done = vec![];
                for (key_a, val_a) in obj_a.iter() {
                    let Some(val_b) = obj_b.get(key_a) else {
                        return std::cmp::Ordering::Greater;
                    };
                    let ord = compare_tera_values(Some(val_a), Some(val_b));
                    if ord.is_ne() {
                        return ord;
                    }
                    keys_done.push(key_a.clone());
                }
                if !obj_b.keys().all(|k| keys_done.contains(k)) {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Equal
                }
            }
            _ => std::cmp::Ordering::Greater,
        },
    }
}

fn deserialize_id<'de, D>(deser: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    log::debug!("Custom deserialize");
    let val = serde_json::Value::deserialize(deser)?;
    log::debug!("Got id: {val}");
    Ok(val.as_u64().unwrap_or(0))
}
