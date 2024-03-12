use serde::{Deserialize, Serialize};
use tera::Context;

use crate::errors::Errcode;

use super::{Storage, StorageQuery};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "query", content = "args")]
#[serde(rename_all = "snake_case")]
pub enum ContextQuery {
    Plain(serde_json::Value),
    RecentPages(String, usize),
    // TODO    Add all cases of context queries
    // - Tags
    // - Series
}

impl ContextQuery {
    pub async fn insert_context(
        &self,
        storage: &Storage,
        name: &String,
        ctxt: &mut Context,
    ) -> Result<(), Errcode> {
        match self {
            ContextQuery::Plain(d) => ctxt.insert(name, d),
            ContextQuery::RecentPages(ptype, nb) => {
                let val = storage
                    .query(StorageQuery::recent_pages(&ptype, *nb))
                    .await
                    .recent_pages()?;
                ctxt.insert(name, &val);
            }
        }
        Ok(())
    }
}
