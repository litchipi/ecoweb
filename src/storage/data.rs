use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::errors::Errcode;
use crate::page::PageMetadata;

use super::StorageErrorType;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum StorageData {
    Nothing,
    RecentPages(Vec<PageMetadata>),
    SimilarPages(Vec<PageMetadata>),
    QueryMetadata(Vec<serde_json::Value>),
    PageContent {
        metadata: PageMetadata,
        body: String,
        lang: Option<String>,
    },
    Templates(HashMap<String, String>),
    StaticFileData(Vec<u8>),
    Error(StorageErrorType),
    Context(toml::Value),
}

impl StorageData {
    #[inline]
    pub fn query_metadata(self) -> Result<Vec<serde_json::Value>, Errcode> {
        match self {
            StorageData::QueryMetadata(val) => Ok(val),
            StorageData::Error(e) => Err(Errcode::StorageError(e)),
            _ => Err(Errcode::WrongStorageData("QueryMetadata")),
        }
    }

    #[inline]
    pub fn similar_pages(self) -> Result<Vec<PageMetadata>, Errcode> {
        match self {
            StorageData::SimilarPages(pages) => Ok(pages),
            StorageData::Error(e) => Err(Errcode::StorageError(e)),
            _ => Err(Errcode::WrongStorageData("SimilarPages")),
        }
    }

    #[inline]
    pub fn page_content(self) -> Result<(Option<String>, PageMetadata, String), Errcode> {
        match self {
            StorageData::PageContent {
                metadata,
                body,
                lang,
            } => Ok((lang, metadata, body)),
            StorageData::Error(e) => Err(Errcode::StorageError(e)),
            _ => Err(Errcode::WrongStorageData("PageContent")),
        }
    }

    #[inline]
    pub fn recent_pages(self) -> Result<Vec<PageMetadata>, Errcode> {
        match self {
            StorageData::RecentPages(pages) => Ok(pages),
            StorageData::Error(e) => Err(Errcode::StorageError(e)),
            _ => Err(Errcode::WrongStorageData("RecentPages")),
        }
    }

    #[inline]
    pub fn base_templates(self) -> Result<HashMap<String, String>, Errcode> {
        match self {
            StorageData::Templates(tmap) => Ok(tmap),
            StorageData::Error(e) => Err(Errcode::StorageError(e)),
            _ => Err(Errcode::WrongStorageData("BaseTemplate")),
        }
    }

    #[inline]
    pub fn static_file(self) -> Result<Vec<u8>, Errcode> {
        match self {
            StorageData::StaticFileData(data) => Ok(data),
            StorageData::Error(e) => Err(Errcode::StorageError(e)),
            _ => Err(Errcode::WrongStorageData("StaticFileData")),
        }
    }

    #[inline]
    pub fn context(self) -> Result<toml::Value, Errcode> {
        match self {
            StorageData::Context(data) => Ok(data),
            StorageData::Error(e) => Err(Errcode::StorageError(e)),
            _ => Err(Errcode::WrongStorageData("Context")),
        }
    }
}
