use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::ContextQuery;

#[derive(Debug, Serialize, Deserialize)]
pub struct PageMetadata {
    pub add_context: HashMap<String, ContextQuery>,
}

#[derive(Debug)]
pub enum StorageData {
    RecentPages(Vec<PageMetadata>),
    PageContent {
        metadata: PageMetadata,
        body: String,
    },
    // TODO    Add all cases of query
    // - Page metadata
    // - Series metadata
    // - Pages by tags
    // - Error code
}

impl StorageData {
    pub fn page_content(self) -> Option<(PageMetadata, String)> {
        match self {
            StorageData::PageContent { metadata, body } => Some((metadata, body)),
            _ => None,
        }
    }

    pub fn recent_pages(self) -> Option<Vec<PageMetadata>> {
        match self {
            StorageData::RecentPages(pages) => Some(pages),
            _ => None,
        }
    }
}
