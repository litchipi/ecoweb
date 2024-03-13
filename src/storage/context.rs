use serde::{Deserialize, Serialize};
use tera::Context;

use crate::errors::Errcode;
use crate::page::PageMetadata;

use super::{Storage, StorageQuery};

pub type MetadataQuery = Vec<String>;
pub type MetadataFilter = (Vec<String>, Option<serde_json::Value>);

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "query", content = "args")]
#[serde(rename_all = "snake_case")]
pub enum ContextQuery {
    Plain(serde_json::Value),
    RecentPages(String, usize),
    SimilarPagesFromMetadata(String, MetadataQuery, usize),
    SimilarPagesFromUri(String, MetadataQuery, String, usize),
    QueryMetadata(String, MetadataQuery),
    QueryFilterMetadata(String, MetadataFilter, MetadataQuery),
}

impl ContextQuery {
    // TODO    Factorize as much as possible this code into separate functions
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
                let Some(val) = page_md.get_metadata(keys) else {
                    log::trace!("Not val found for keys {keys:?}");
                    return Ok(());
                };

                let qry = StorageQuery::similar_pages(slug, (keys.clone(), Some(val.clone())));
                let val = storage
                    .query(qry)
                    .await
                    .similar_pages()?;
                ctxt.insert(name, &val);
            },
            ContextQuery::SimilarPagesFromUri(ref slug, ref keys, ref uri_slug, nb) => {
                if keys.is_empty() {
                    return Err(
                        Errcode::ContextQueryBuild("similar_pages_from_metadata", "empty keys".to_string())
                    );
                }
                // TODO IMPORTANT Get match info from args, and get value from it
                // Then build the query for similar pages based on it
                // ctxt.insert(name, &val);
            }
            ContextQuery::QueryMetadata(slug, query) => {
                let qry = StorageQuery::query_metadata(slug, (vec![], None), query.clone());
                let val = storage.query(qry).await.query_metadata()?;
                log::debug!("{name} = {val:?}");
                ctxt.insert(name, &val);
            }
            ContextQuery::QueryFilterMetadata(slug, filter, query) => {
                let qry = StorageQuery::query_metadata(slug, filter.clone(), query.clone());
                let val = storage.query(qry).await.query_metadata()?;
                log::debug!("{name} = {val:?}");
                ctxt.insert(name, &val);
            }
        }
        Ok(())
    }
}
