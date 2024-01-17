use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::config::SiteConfig;
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

impl PostMetadata {
    pub fn to_rss_item(&self, cfg: &SiteConfig, xml: &mut String) {
        *xml += "<item>";
        *xml += format!("<title>{}</title>", self.title).as_str();
        *xml += format!(
            "<link type=\"text/html\" title=\"{}\">{}/post/{}</link>",
            self.title, cfg.base_url, self.id,
        )
        .as_str();

        *xml += format!(
            "<author>{} ({})</author>",
            cfg.author_email, cfg.author_name
        )
        .as_str();
        *xml += format!("<guid isPermaLink=\"false\">{}</guid>", self.id).as_str();

        let date: DateTime<Utc> = DateTime::from_naive_utc_and_offset(
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

fn deser_id<'de, T, D>(de: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    let str_repr = String::deserialize(de)?;
    str_repr.parse().map_err(serde::de::Error::custom)
}
