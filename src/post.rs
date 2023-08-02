use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::Path;

use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::cache::CacheElement;
use crate::errors::Errcode;
use crate::loader::PostFilter;
use crate::render::context::SiteContext;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerieMetadata {
    pub title: String,
    pub description: String,
    #[serde(skip_deserializing)]
    pub slug: String,
    pub end_date: i64, // Seconds since Epoch
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PostMetadata {
    #[serde(skip_deserializing)]
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
    pub images_add_attribute: HashMap<usize, String>,
    #[serde(default)]
    pub tags: Vec<String>,

    #[serde(skip_deserializing)]
    pub content_checksum: u64,

    #[serde(skip_deserializing)]
    last_sync: Option<i64>, // Last time the related posts were synced
}

impl PostMetadata {
    pub fn compute_checksum(&mut self, content: &String) {
        let mut s = DefaultHasher::new();
        content.hash(&mut s);
        self.content_checksum = s.finish();
    }

    pub fn read_from_file(f: &Path) -> Result<PostMetadata, Errcode> {
        let file = std::fs::File::open(f)?;
        let mut metadata: PostMetadata = serde_json::from_reader(file)?;
        metadata.compute_id();
        metadata.validate()?;
        Ok(metadata)
    }

    pub fn validate(&self) -> Result<(), Errcode> {
        if let Some(m) = self.modified {
            // Modified before published
            if m < self.date {
                return Err(Errcode::MetadataValidationFailed("post", "mod_date < date"));
            }
        }
        Ok(())
    }

    pub fn compute_id(&mut self) {
        let mut s = DefaultHasher::new();
        self.title.hash(&mut s);
        self.serie.hash(&mut s);
        self.category.hash(&mut s);
        self.id = s.finish();
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

impl CacheElement for PostMetadata {
    fn len(&self) -> usize {
        let mut sz = std::mem::size_of::<u64>();
        sz += self.title.len();
        if let Some(ref d) = self.description {
            sz += d.len();
        }
        if let Some(ref c) = self.category {
            sz += c.len();
        }
        if let Some(ref s) = self.serie {
            sz += s.len();
        }
        if let Some(ref st) = self.serie_title {
            sz += st.len();
        }
        sz += std::mem::size_of::<i64>();
        if self.modified.is_some() {
            sz += std::mem::size_of::<i64>();
        }
        for val in self.images_add_attribute.values() {
            sz += std::mem::size_of::<usize>();
            sz += val.len();
        }
        for tag in self.tags.iter() {
            sz += tag.len();
        }
        sz
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Post {
    pub metadata: PostMetadata,
    pub content: String,
}

impl CacheElement for Post {
    fn len(&self) -> usize {
        self.metadata.len() + self.content.len()
    }
}

impl PostMetadata {
    pub fn to_rss_item(&self, ctxt: &SiteContext, xml: &mut String) {
        *xml += "<item>";
        *xml += format!("<title>{}</title>", self.title).as_str();
        *xml += format!(
            "<link type=\"text/html\" title=\"{}\">{}/post/{}</link>",
            self.title, ctxt.base_url, self.id,
        )
        .as_str();

        *xml += format!(
            "<author>{} ({})</author>",
            ctxt.author_email, ctxt.author_name
        )
        .as_str();
        *xml += format!("<guid isPermaLink=\"false\">{}</guid>", self.id).as_str();

        let date: DateTime<Utc> = DateTime::from_utc(
            NaiveDateTime::from_timestamp_opt(self.date, 0).unwrap(),
            Utc,
        );
        *xml += format!("<pubDate>{}</pubDate>", date.to_rfc2822()).as_str();

        if let Some(ref d) = self.description {
            *xml += format!("<description type=\"html\">{}</description>", d).as_str();
        } else {
            *xml += "<description>No description available</description>";
        }

        if let Some(ref c) = self.category {
            *xml += format!("<category>{}</category>", c).as_str();
        }
        for tag in self.tags.iter() {
            *xml += format!("<category>{}</category>", tag).as_str();
        }
        *xml += "</item>";
    }
}
