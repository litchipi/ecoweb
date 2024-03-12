use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::errors::Errcode;
use crate::page::PageMetadata;

use super::StorageErrorType;

#[derive(Clone, Serialize, Deserialize, Debug)]
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
    Template(String),
    BaseTemplate(HashMap<String, String>),
    Error(StorageErrorType),
}

impl StorageData {
    pub fn page_content(self) -> Result<(PageMetadata, String), Errcode> {
        match self {
            StorageData::PageContent { metadata, body } => Ok((metadata, body)),
            StorageData::Error(e) => Err(Errcode::StorageError(e)),
            _ => Err(Errcode::WrongStorageData("PageContent")),
        }
    }

    pub fn recent_pages(self) -> Result<Vec<PageMetadata>, Errcode> {
        match self {
            StorageData::RecentPages(pages) => Ok(pages),
            StorageData::Error(e) => Err(Errcode::StorageError(e)),
            _ => Err(Errcode::WrongStorageData("RecentPages")),
        }
    }

    pub fn template(self) -> Result<String, Errcode> {
        match self {
            StorageData::Template(template_str) => Ok(template_str),
            StorageData::Error(e) => Err(Errcode::StorageError(e)),
            _ => Err(Errcode::WrongStorageData("Template")),
        }
    }

    pub fn base_templates(self) -> Result<HashMap<String, String>, Errcode> {
        match self {
            StorageData::BaseTemplate(tmap) => Ok(tmap),
            StorageData::Error(e) => Err(Errcode::StorageError(e)),
            _ => Err(Errcode::WrongStorageData("BaseTemplate")),
        }
    }
}
