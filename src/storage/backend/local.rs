use std::collections::HashMap;
use std::path::PathBuf;

use actix_web::{HttpResponse, HttpResponseBuilder};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::page::PageMetadata;
use crate::scss::{setup_css, ScssError};
use crate::storage::query::StorageQueryMethod;
use crate::storage::{StorageData, StorageQuery};

use super::StorageBackend;

fn canonicalize_to_root(path: &mut PathBuf, root: &PathBuf) -> Result<(), LocalStorageError> {
    *path = root
        .join(&path)
        .canonicalize()
        .map_err(|e| LocalStorageError::InitPaths(format!("{path:?}: {e:?}")))?;
    Ok(())
}

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
    CreateDir(String),
    InitPaths(String),
    ScssProcess(ScssError),
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
    // Data
    data_root: PathBuf,
    supported_lang: Vec<String>,

    // Templates
    template_root: PathBuf,
    base_templates: Vec<String>,

    // Assets
    include_assets: Vec<PathBuf>,

    // CSS
    css_output_dir: PathBuf,
    scss: HashMap<String, Vec<PathBuf>>,
    scss_root: PathBuf,
}

impl LocalStorage {
    pub fn canonicalize_paths(&mut self, config: &Config) -> Result<(), LocalStorageError> {
        canonicalize_to_root(&mut self.data_root, &config.root)?;
        canonicalize_to_root(&mut self.template_root, &config.root)?;
        std::fs::create_dir_all(config.root.join(&self.css_output_dir))
            .map_err(|e| LocalStorageError::CreateDir(format!("css: {e:?}")))?;
        canonicalize_to_root(&mut self.css_output_dir, &config.root)?;
        self.include_assets.push(self.css_output_dir.clone());
        for inc in self.include_assets.iter_mut() {
            *inc = config
                .root
                .join(&inc)
                .canonicalize()
                .map_err(|e| LocalStorageError::InitPaths(format!("{inc:?}: {e:?}")))?;
        }
        Ok(())
    }

    pub fn get_content_path(
        &self,
        qry: &StorageQuery,
        tail: Vec<&str>,
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
            path.push(t.to_string());
        }
        path.set_extension("md");
        log::debug!("Path: {path:?}");
        Ok(path)
    }

    pub fn load_static_file(&self, path: PathBuf) -> Result<StorageData, LocalStorageError> {
        let data =
            std::fs::read(path).map_err(|e| LocalStorageError::LoadStaticFile(e.to_string()))?;
        // TODO    Catch javascript files here, pass through minification process
        Ok(StorageData::StaticFileData(data))
    }

    pub fn load_content(&self, path: PathBuf) -> Result<StorageData, LocalStorageError> {
        if !path.exists() {
            return Err(LocalStorageError::DataNotFound(path));
        }

        let content = std::fs::read_to_string(path.clone())
            .map_err(|e| LocalStorageError::LoadContent(format!("{e:?}")))?;

        let mut split = content.split("---");
        let metadata = split.next().unwrap();
        let body = split
            .next()
            .ok_or(LocalStorageError::NoMetadataSplit)?
            .to_string();

        let mut metadata: PageMetadata = toml::from_str(metadata)
            .map_err(|e| LocalStorageError::MetadataDecode(format!("{e:?}")))?;
        if metadata.id == 0 {
            metadata.update_id(path.into_os_string().to_string_lossy().to_string());
        }

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
                // TODO    FIXME    Get post file path from ID
                let path = self.get_content_path(&qry, vec![format!("{id}").as_str()])?;
                self.load_content(path)
            }
            StorageQueryMethod::ContentSlug(ref slug) => {
                let path = self.get_content_path(&qry, vec![slug])?;
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
                let fpath = PathBuf::from(f.trim_start_matches("/"));
                let parent = fpath
                    .components()
                    .nth(0)
                    .ok_or(LocalStorageError::BadRequest(
                        "static file no /".to_string(),
                    ))?;
                for inc in self.include_assets.iter() {
                    let last = inc.components().last().unwrap();
                    if last == parent {
                        let path = inc.join(fpath.strip_prefix(parent).unwrap());
                        let path = path
                            .canonicalize()
                            .map_err(|e| LocalStorageError::DataNotFound(path))?;
                        if path.starts_with(inc) {
                            return self.load_static_file(path);
                        }
                    }
                }
                Err(LocalStorageError::DataNotFound(fpath))
            }
            StorageQueryMethod::GetSimilarPages(keys, val) => {
                // TODO    Get similar pages based on metadata key and value
                Ok(StorageData::SimilarPages(vec![]))
            }
            StorageQueryMethod::QueryMetadata(filter, query) => {
                // TODO   IMPORTANT  Filter Metadata of all pages, returns the query
                Ok(StorageData::Nothing)
            }
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
        storage.canonicalize_paths(config)?;
        setup_css(
            config.root.join(&storage.scss_root), &storage.scss, &storage.css_output_dir)
            .map_err(|e| LocalStorageError::ScssProcess(e))?;
        Ok(storage)
    }

    async fn has_changed(&self, qry: &StorageQuery) -> bool {
        // TODO    Find a way to know when posts are changed or not
        false
    }

    async fn query(&self, qry: StorageQuery) -> StorageData {
        match self.dispatch(qry) {
            Ok(data) => data,
            Err(e) => StorageData::Error(e),
        }
    }
}
