use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::errors::Errcode;
use crate::page::PageMetadata;
use crate::storage::query::StorageQueryMethod;
use crate::storage::{StorageData, StorageQuery};

use super::StorageBackend;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalStorage {
    data_root: PathBuf,
    supported_lang: Vec<String>,
    template_root: PathBuf,
}

impl LocalStorage {
    pub fn get_content_path(&self, qry: &StorageQuery, tail: Vec<String>) -> Result<PathBuf, Errcode> {
        let mut path = self.data_root.join(qry.storage_slug.clone());
        if let Some(ref lang) = qry.lang_pref {
            for l in lang.iter() {
                if self.supported_lang.contains(l) {
                    path.push(l);
                    break;
                }
            }
            return Err(Errcode::LangNotSupported(lang.clone()));
        }
        for t in tail {
            path.push(t);
        }
        path.set_extension("md");
        log::debug!("Path: {path:?}");
        Ok(path)
    }

    pub fn load_content(&self, path: PathBuf) -> Result<StorageData, Errcode> {
        if !path.exists() {
            return Err(Errcode::DataNotFound(path.as_path().to_string_lossy().to_string()));
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e|
                Errcode::FilesystemError("storage-local::load_content", Arc::new(e))
            )?;

        let mut split = content.split("---");
        let metadata = split.next()
            .ok_or(Errcode::ContentMalformed("storage-local::metadata::split"))?;
        let metadata: PageMetadata = toml::from_str(metadata)
            .map_err(|e| {
                log::error!("Parse metadata from post: {e:?}");
                Errcode::ContentMalformed("storage-local::metadata::toml-parse")
            })?;
        let body = split.next()
            .ok_or(Errcode::ContentMalformed("storage-local::body::split"))?
            .to_string();
        Ok(StorageData::PageContent {
            metadata,
            body,
        })
    }

    pub fn dispatch(&self, qry: StorageQuery) -> Result<StorageData, Errcode> {
        match qry.method {
            StorageQueryMethod::NoOp => {
                log::debug!("Local storage No Op");
                Ok(StorageData::Nothing)
            },
            StorageQueryMethod::ContentNoId => {
                let path = self.get_content_path(&qry, vec![])?;
                self.load_content(path)
            },
            StorageQueryMethod::ContentNumId(id) => {
                let path = self.get_content_path(&qry, vec![format!("{id}")])?;
                self.load_content(path)
            },
            StorageQueryMethod::GetRecentPages => {
                // TODO    Get recent posts from local filesystem
                Ok(StorageData::RecentPages(vec![]))
            },
        }
    }
}

impl StorageBackend for LocalStorage {
    fn init(config: &Config) -> Self where Self: Sized {
        config.local_storage.clone()
    }

    async fn has_changed(&self, qry: &StorageQuery) -> bool {
        // TODO    Find a way to know when posts are changed or not
        false
    }

    async fn query(&self, qry: StorageQuery) -> StorageData {
        match self.dispatch(qry) {
            Ok(data) => {
                log::debug!("Local storage data: {data:?}");
                data
            },
            Err(e) => StorageData::Error(e),
        }
    }
}
