use crate::errors::Errcode;
use crate::page::PageMetadata;

#[derive(Clone, Debug)]
pub enum StorageData {
    Nothing,
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
    Error(Errcode),
}

impl StorageData {
    pub fn page_content(self) -> Result<(PageMetadata, String), Errcode> {
        match self {
            StorageData::PageContent { metadata, body } => Ok((metadata, body)),
            StorageData::Error(e) => Err(e),
            _ => Err(Errcode::WrongStorageData("PageContent")),
        }
    }

    pub fn recent_pages(self) -> Result<Vec<PageMetadata>, Errcode> {
        match self {
            StorageData::RecentPages(pages) => Ok(pages),
            StorageData::Error(e) => Err(e),
            _ => Err(Errcode::WrongStorageData("RecentPages")),
        }
    }
}
