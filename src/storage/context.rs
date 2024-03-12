use serde::{Deserialize, Serialize};
use tera::Context;

use crate::errors::Errcode;
use crate::page::PageMetadata;

use super::{Storage, StorageQuery};

pub type MetadataQuery = Vec<String>;
pub type MetadataFilter = Vec<String>;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "query", content = "args")]
#[serde(rename_all = "snake_case")]
pub enum ContextQuery {
    Plain(serde_json::Value),
    RecentPages(String, usize),
    SimilarPagesFromMetadata(String, MetadataFilter, usize),
    QueryMetadata(String, String, MetadataFilter, MetadataQuery),
}

impl ContextQuery {
    pub async fn insert_context(
        &self,
        storage: &Storage,
        name: &String,
        page_md: &PageMetadata,
        ctxt: &mut Context,
    ) -> Result<(), Errcode> {
        match self {
            ContextQuery::Plain(d) => ctxt.insert(name, d),
            ContextQuery::RecentPages(ref slug, nb) => {
                let val = storage
                    .query(StorageQuery::recent_pages(slug, *nb))
                    .await
                    .recent_pages()?;
                ctxt.insert(name, &val);
            }
            ContextQuery::SimilarPagesFromMetadata(ref slug, ref keys, nb) => {
                if keys.is_empty() {
                    return Err(
                        Errcode::ContextQueryBuild("similar_pages_from_metadata", "empty keys".to_string())
                    );
                }
                let mut keys_iter = keys.iter();
                let mut val = page_md.metadata.get(keys_iter.next().unwrap());
                for key in keys_iter {
                    if let Some(data) = val {
                        if data.is_object() {
                            val = data.as_object().unwrap().get(key);
                            continue;
                        }
                    }
                    log::trace!("Unable to build context query: {key} not contained in map");
                    return Ok(());
                }
                let Some(val) = val else {
                    log::trace!("Unable to build context query: value for this key is empty");
                    return Ok(());
                };

                let val = storage
                    .query(StorageQuery::similar_pages(slug, keys.clone(), val.clone()))
                    .await
                    .similar_pages()?;
                ctxt.insert(name, &val);
            },
            ContextQuery::QueryMetadata(slug, name, filter, queries) => {
                let qry = StorageQuery::query_metadata(slug, filter.clone(), queries.clone());
                let val = storage.query(qry).await.query_metadata()?;
                ctxt.insert(name, &val);
            }
        }
        Ok(())
    }
}
