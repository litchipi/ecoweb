use serde::{Deserialize, Serialize};
use tera::Context;

use crate::errors::Errcode;

use super::{Storage, StorageQuery};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ContextQuery {
    Plain(serde_json::Value),
    RecentPages(String, usize),
    // TODO    Add all cases of context queries
    // - Tags
    // - Series
}

impl ContextQuery {
    pub fn insert_context(
        &self,
        ldr: &Storage,
        name: &String,
        ctxt: &mut Context,
    ) -> Result<(), Errcode> {
        match self {
            ContextQuery::Plain(d) => ctxt.insert(name, d),
            ContextQuery::RecentPages(ptype, nb) => {
                let val = ldr
                    .query(&StorageQuery::recent_pages(&ptype, *nb))
                    .recent_pages()
                    .ok_or(Errcode::NoRecentPagesFound(ptype.clone()))?;
                ctxt.insert(name, &val);
            }
        }
        Ok(())
    }
}
