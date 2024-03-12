use std::collections::HashMap;
use std::path::PathBuf;

use actix_web::{HttpResponse, HttpResponseBuilder};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::page::PageMetadata;
use crate::storage::query::StorageQueryMethod;
use crate::storage::{StorageData, StorageQuery};

use super::StorageBackend;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum LocalStorageError {
    LangNotSupported(Vec<String>),
    TemplateNotFound(PathBuf),
    DataNotFound(PathBuf),
    LoadContent(String),
    LoadStaticFile(String),
    MetadataDecode(String),
    NoMetadataSplit,
    BadRequest(String),
    IncludeCanonicalize(String),
}

impl Into<HttpResponseBuilder> for LocalStorageError {
    fn into(self) -> HttpResponseBuilder {
        match self {
            LocalStorageError::DataNotFound(_) => HttpResponse::NotFound(),
            _ => HttpResponse::InternalServerError(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalStorage {
    data_root: PathBuf,
    supported_lang: Vec<String>,
    template_root: PathBuf,
    base_templates: Vec<String>,
    include_assets: Vec<PathBuf>,
}

impl LocalStorage {
    pub fn get_content_path(
        &self,
        qry: &StorageQuery,
        tail: Vec<String>,
    ) -> Result<PathBuf, LocalStorageError> {
        let mut path = self.data_root.join(qry.storage_slug.clone());
        if let Some(ref lang) = qry.lang_pref {
            for l in lang.iter() {
                if self.supported_lang.contains(l) {
                    path.push(l);
                    break;
                }
            }
            return Err(LocalStorageError::LangNotSupported(lang.clone()));
        }
        for t in tail {
            path.push(t);
        }
        path.set_extension("md");
        log::debug!("Path: {path:?}");
        Ok(path)
    }

    pub fn load_static_file(&self, path: PathBuf) -> Result<StorageData, LocalStorageError> {
        let data = std::fs::read(path)
            .map_err(|e| LocalStorageError::LoadStaticFile(e.to_string()))?;
        Ok(StorageData::StaticFileData(data))
    }

    pub fn load_content(&self, path: PathBuf) -> Result<StorageData, LocalStorageError> {
        if !path.exists() {
            return Err(LocalStorageError::DataNotFound(path));
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| LocalStorageError::LoadContent(format!("{e:?}")))?;

        let mut split = content.split("---");
        let metadata = split.next().unwrap();
        let body = split
            .next()
            .ok_or(LocalStorageError::NoMetadataSplit)?
            .to_string();
        let metadata: PageMetadata = toml::from_str(metadata)
            .map_err(|e| LocalStorageError::MetadataDecode(format!("{e:?}")))?;
        Ok(StorageData::PageContent { metadata, body })
    }

    pub fn load_template(&self, name: &String) -> Result<String, LocalStorageError> {
        let path = self.template_root.join(name);
        if !path.exists() {
            return Err(LocalStorageError::TemplateNotFound(path));
        }
        let content = std::fs::read_to_string(path)
            .map_err(|e| LocalStorageError::LoadContent(format!("{e:?}")))?;
        Ok(content)
    }

    pub fn dispatch(&self, qry: StorageQuery) -> Result<StorageData, LocalStorageError> {
        match qry.method {
            StorageQueryMethod::NoOp => {
                log::debug!("Local storage No Op");
                Ok(StorageData::Nothing)
            }
            StorageQueryMethod::ContentNoId => {
                let path = self.get_content_path(&qry, vec![])?;
                self.load_content(path)
            }
            StorageQueryMethod::ContentNumId(id) => {
                let path = self.get_content_path(&qry, vec![format!("{id}")])?;
                self.load_content(path)
            }
            StorageQueryMethod::RecentPages => {
                // TODO    Get recent posts from local filesystem
                Ok(StorageData::RecentPages(vec![]))
            }
            StorageQueryMethod::PageTemplate(name) => {
                let data = self.load_template(&name)?;
                Ok(StorageData::Template(data))
            }
            StorageQueryMethod::BaseTemplates => {
                let mut base_templates = HashMap::new();
                for template in self.base_templates.iter() {
                    let data = self.load_template(template)?;
                    base_templates.insert(template.clone(), data);
                }
                Ok(StorageData::BaseTemplate(base_templates))
            }
            StorageQueryMethod::StaticFile(f) => {
                let fpath = PathBuf::from(&f);
                let parent = fpath
                    .components().nth(0)
                    .ok_or(
                        LocalStorageError::BadRequest("static file no /".to_string())
                    )?;
                for inc in self.include_assets.iter() {
                    let last = inc.components().last().unwrap();
                    if last == parent {
                        let path = inc.join(fpath.strip_prefix(parent).unwrap());
                        let path = path.canonicalize()
                            .map_err(|e| LocalStorageError::DataNotFound(path))?;
                        if path.starts_with(inc) {
                            return self.load_static_file(path);
                        }
                    }
                }
                Err(LocalStorageError::DataNotFound(fpath))
            },
        }
    }
}

impl StorageBackend for LocalStorage {
    type Error = LocalStorageError;
    
    fn init(config: &Config) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let mut storage = config.local_storage.clone();
        for inc in storage.include_assets.iter_mut() {
            *inc = inc.canonicalize()
                .map_err(|e| LocalStorageError::IncludeCanonicalize(e.to_string()))?;
        }
        Ok(storage)
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
            }
            Err(e) => StorageData::Error(e),
        }
    }
}
