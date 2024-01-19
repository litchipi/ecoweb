use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};

use crate::loader::PostFilter;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerieMetadata {
    #[serde(skip_deserializing)]
    pub slug: String,
    pub title: String,
    pub description: String,
    pub end_date: i64, // Seconds since Epoch
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PostMetadata {
    #[serde(default, deserialize_with = "deser_id")]
    pub id: u64,
    pub title: String,
    pub description: Option<String>,

    pub category: Option<String>,
    pub serie: Option<String>,
    #[serde(skip_deserializing)]
    pub serie_title: Option<String>,

    pub date: i64, // Seconds since Epoch
    pub modified: Option<i64>,

    #[serde(default)]
    pub tags: Vec<String>,

    #[serde(default)]
    pub hidden: bool,
}

impl PostMetadata {
    pub fn compute_id(&mut self) {
        if self.id == 0 {
            let mut s = DefaultHasher::new();
            self.title.hash(&mut s);
            self.serie.hash(&mut s);
            self.category.hash(&mut s);
            self.id = s.finish();
        }
    }

    pub fn filter(&self, filter: &PostFilter) -> bool {
        match filter {
            PostFilter::NoFilter => true,
            PostFilter::NoSerie => self.serie.is_none(),
            PostFilter::DifferentThan(id) => self.id != *id,
            PostFilter::Serie(ref s) => {
                if let Some(ref serie) = self.serie {
                    serie == s
                } else {
                    false
                }
            }
            PostFilter::Category(ref c) => {
                if let Some(ref category) = self.category {
                    category == c
                } else {
                    false
                }
            }
            PostFilter::ContainsTag(tag) => self.tags.contains(tag),
            PostFilter::Combine(all) => all.iter().all(|f| self.filter(f)),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Post {
    pub metadata: PostMetadata,
    pub content: String,
    pub post_nav: String,
}

fn deser_id<'de, T, D>(de: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    let str_repr = String::deserialize(de)?;
    str_repr.parse().map_err(serde::de::Error::custom)
}
