use serde::{Deserialize, Serialize};
use tera::Context;

use crate::{errors::Errcode, routes::RequestArgs};
use crate::page::PageMetadata;

use super::query::QueryListOptions;
use super::{StorageData, StorageQuery};

pub type MetadataQuery = Vec<String>;
pub type MetadataFilter = (Vec<String>, Option<serde_json::Value>);

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "query", content = "args")]
#[serde(rename_all = "snake_case")]
pub enum ContextQuery {
    Plain(serde_json::Value),
    RecentPages(String, QueryListOptions),
    SimilarPagesFromMetadata(String, MetadataQuery, QueryListOptions),
    SimilarPagesFromUri(String, MetadataQuery, String, QueryListOptions),
    QueryMetadata(String, MetadataQuery),
    QueryFilterMetadata(String, MetadataFilter, MetadataQuery),
}

impl ContextQuery {
    pub fn insert_data(&self, name: &String, ctxt: &mut Context, data: StorageData) -> Result<(), Errcode> {
            match self {
                ContextQuery::RecentPages(_, _) => ctxt.insert(name, &data.recent_pages()?),
                ContextQuery::SimilarPagesFromMetadata(_, _, _) => ctxt.insert(name, &data.similar_pages()?),
                ContextQuery::SimilarPagesFromUri(_, _, _, _) => ctxt.insert(name, &data.similar_pages()?),
                ContextQuery::QueryMetadata(_, _) => ctxt.insert(name, &data.query_metadata()?),
                ContextQuery::QueryFilterMetadata(_, _, _) => ctxt.insert(name, &data.query_metadata()?),
                _ => {},
            }
            Ok(())
    }

    pub fn get_storage_query(&self, args: &RequestArgs, page_md: &PageMetadata) -> Result<Option<StorageQuery>, Errcode> {
        match self {
            ContextQuery::Plain(d) => Ok(None),
            ContextQuery::RecentPages(ref slug, opts) => {
                Ok(Some(StorageQuery::recent_pages(slug, opts)))
            }
            ContextQuery::SimilarPagesFromMetadata(ref slug, ref keys, opts) => {
                if keys.is_empty() {
                    return Err(
                        Errcode::ContextQueryBuild("similar_pages_from_metadata", "empty keys".to_string())
                    );
                }
                let Some(val) = page_md.get_metadata(keys) else {
                    log::trace!("Not val found for keys {keys:?}");
                    return Ok(None);
                };

                let qry = StorageQuery::similar_pages(
                    slug,
                    (keys.clone(), Some(val.clone())),
                    opts,
                );
                Ok(Some(qry))
            }
            ContextQuery::SimilarPagesFromUri(ref slug, ref keys, ref uri_slug, opts) => {
                if keys.is_empty() {
                    return Err(
                        Errcode::ContextQueryBuild("similar_pages_from_uri", "empty keys".to_string())
                    );
                }
                let val = args.get_query_slug(uri_slug)?;
                let qry = StorageQuery::similar_pages(
                    slug,
                    (keys.clone(), Some(val.into())),
                    opts,
                );
                Ok(Some(qry))
            }
            ContextQuery::QueryMetadata(slug, query) => {
                Ok(Some(StorageQuery::query_metadata(slug, (vec![], None), query.clone())))
            }
            ContextQuery::QueryFilterMetadata(slug, filter, query) => {
                Ok(Some(StorageQuery::query_metadata(slug, filter.clone(), query.clone())))
            }
        }
    }
}
