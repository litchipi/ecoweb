use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use actix_web::{HttpResponse, HttpResponseBuilder};
use parking_lot::RwLock;
use path_absolutize::Absolutize;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::page::PageMetadata;
use crate::scss::{compile_scss, ScssError};
use crate::storage::query::StorageQueryMethod;
use crate::storage::{StorageData, StorageQuery};

use super::StorageBackend;

const SPLIT_PAT: &str = "---";

fn canonicalize_to_root(path: &mut PathBuf, root: &Path) -> Result<(), LocalStorageError> {
    *path = root
        .join(&path)
        .canonicalize()
        .map_err(|e| LocalStorageError::InitPaths(format!("{path:?}: {e:?}")))?;
    Ok(())
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum LocalStorageError {
    LoadContent(String),
    LoadStaticFile(String),
    LoadContext(String),

    DataNotFound(PathBuf),
    TemplateLoading(String),

    NoMatch(String),
    TooManyMatches(usize, usize),

    TomlDecode(String),
    NoMetadataSplit,

    BadRequest(String),

    InitPaths(String),
    CreateDir(String),
    NotDataDir(PathBuf),
    ListFiles(String),
    ListFilesPathUnwrap(String),

    CssNotFound(String),
    ScssProcess(ScssError),

    AttackSuspected(String),
}

impl From<LocalStorageError> for HttpResponseBuilder {
    fn from(val: LocalStorageError) -> Self {
        match val {
            LocalStorageError::DataNotFound(_) => HttpResponse::NotFound(),
            _ => HttpResponse::InternalServerError(),
        }
    }
}

impl From<ScssError> for LocalStorageError {
    fn from(value: ScssError) -> Self {
        LocalStorageError::ScssProcess(value)
    }
}

//                        Storage    Filepath   Metadata
type PageCache = HashMap<String, Vec<(PathBuf, PageMetadata)>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalStorage {
    #[serde(skip)]
    all_pages: Arc<RwLock<PageCache>>,

    // Data
    data_root: PathBuf,
    supported_lang: Vec<String>,
    default_sort: (Vec<String>, bool),

    // Templates
    template_root: PathBuf,
    base_templates: Vec<String>,

    // Assets
    include_assets: Vec<PathBuf>,

    // CSS
    scss: HashMap<String, Vec<PathBuf>>,
    scss_root: PathBuf,
}

impl LocalStorage {
    pub fn canonicalize_paths(&mut self, config: &Config) -> Result<(), LocalStorageError> {
        canonicalize_to_root(&mut self.data_root, &config.root)?;
        canonicalize_to_root(&mut self.template_root, &config.root)?;
        canonicalize_to_root(&mut self.scss_root, &config.root)?;
        for inc in self.include_assets.iter_mut() {
            *inc = config
                .root
                .join(&inc)
                .canonicalize()
                .map_err(|e| LocalStorageError::InitPaths(format!("{inc:?}: {e:?}")))?;
        }
        Ok(())
    }

    pub fn load_css(&self, css: &str) -> Result<StorageData, LocalStorageError> {
        let mut css_content = String::new();
        let Some(scss_files) = self.scss.get(css) else {
            return Err(LocalStorageError::CssNotFound(css.to_string()));
        };

        for scss_file in scss_files {
            css_content += format!("\n/* {scss_file:?} */\n").as_str();
            css_content += compile_scss(self.scss_root.join(scss_file))?.as_str();
            css_content += "\n";
        }

        #[cfg(feature = "css_minify")]
        {
            css_content = minifier::css::minify(&css_content)
                .map_err(|e| ScssError::MinificationError(e.to_string()))?
                .to_string();
        }
        Ok(StorageData::StaticFileData(css_content.as_bytes().to_vec()))
    }

    pub fn load_static_file(&self, path: PathBuf) -> Result<StorageData, LocalStorageError> {
        #[allow(unused_mut)]
        let mut data =
            std::fs::read(&path).map_err(|e| LocalStorageError::LoadStaticFile(e.to_string()))?;

        if let Some(ext) = path.extension() {
            #[cfg(feature = "js_minify")]
            if ext == "js" {
                data = minify_js(data);
            }
        }

        Ok(StorageData::StaticFileData(data))
    }

    pub fn get_content_path(
        &self,
        qry: &StorageQuery,
        name: Option<&str>,
        lang: Option<&String>,
        ext: Option<&str>,
    ) -> Result<PathBuf, LocalStorageError> {
        let mut path = self.data_root.join(qry.storage_slug.clone());
        if let Some(lang) = self.select_lang(qry)? {
            path.push(lang);
        }
        if let Some(name) = name {
            path.push(name);
        }
        if let Some(ext) = ext {
            path.set_extension(ext);
        }
        Ok(path)
    }

    pub fn load_content(&self, path: &Path) -> Result<(PageMetadata, String), LocalStorageError> {
        if !path.exists() {
            return Err(LocalStorageError::DataNotFound(path.to_path_buf()));
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| LocalStorageError::LoadContent(format!("{e:?}")))?;

        if !content.contains(SPLIT_PAT) {
            return Err(LocalStorageError::LoadContent(format!(
                "Split {SPLIT_PAT} not found in {path:?}"
            )));
        }

        let mut split = content.split(SPLIT_PAT);
        let metadata = split.next().unwrap();

        let body = split.collect::<Vec<&str>>().join(SPLIT_PAT).to_string();

        let mut metadata: PageMetadata = toml::from_str(metadata)
            .map_err(|e| LocalStorageError::TomlDecode(format!("{e:?}")))?;
        if metadata.id == 0 {
            metadata.update_id(path.to_string_lossy().to_string());
        }

        Ok((metadata, body))
    }

    fn all_pages_in_dir(
        &self,
        dirpath: &Path,
    ) -> Result<Vec<(PathBuf, PageMetadata)>, LocalStorageError> {
        let all_paths = std::fs::read_dir(dirpath)
            .map_err(|e| LocalStorageError::ListFiles(format!("{e:?}")))?;

        let mut all_pages = vec![];
        for path in all_paths {
            let path = path
                .map_err(|e| LocalStorageError::ListFilesPathUnwrap(format!("{e:?}")))?
                .path();
            if path.is_file() {
                let (metadata, _) = self.load_content(&path)?;
                all_pages.push((path, metadata));
            } else if path.is_dir() {
                all_pages.extend(self.all_pages_in_dir(&path)?);
            }
        }
        Ok(all_pages)
    }

    pub fn register_all_pages(&self, slug: &String) -> Result<(), LocalStorageError> {
        let dirpath = self.data_root.join(slug);
        if !dirpath.is_dir() {
            return Err(LocalStorageError::NotDataDir(dirpath));
        }
        let all_pages = self.all_pages_in_dir(&dirpath)?;
        log::debug!("Registered {} pages in {slug}", all_pages.len());
        self.all_pages.write().insert(slug.clone(), all_pages);
        Ok(())
    }

    pub fn ensure_all_pages_loaded(&self, slug: &String) -> Result<(), LocalStorageError> {
        let pages_reg = self.all_pages.read().contains_key(slug);
        let hot_reload = false;

        #[cfg(feature = "hot-reloading")]
        let hot_reload = true;

        if !pages_reg || hot_reload {
            self.register_all_pages(slug)?;
        }

        Ok(())
    }

    // TODO Create separate functions for each
    pub fn dispatch(&self, qry: StorageQuery) -> Result<StorageData, LocalStorageError> {
        let (sort_key, rev) = if let Some((ref sort_key, rev)) = qry.sort_by {
            (sort_key, rev)
        } else {
            (&self.default_sort.0, self.default_sort.1)
        };
        let lang = self.select_lang(&qry)?;

        match qry.method {
            StorageQueryMethod::NoOp => {
                log::debug!("Local storage No Op");
                Ok(StorageData::Nothing)
            }

            StorageQueryMethod::ContentFromName(ref name) => {
                let path = self.get_content_path(&qry, Some(name), lang.as_ref(), Some("md"))?;
                let (metadata, body) = self.load_content(&path)?;
                Ok(StorageData::PageContent {
                    metadata,
                    body,
                    lang,
                })
            }

            StorageQueryMethod::ContentNumId(id) => {
                self.ensure_all_pages_loaded(&qry.storage_slug)?;
                let all_pages = self.all_pages.read();
                let pages = all_pages.get(&qry.storage_slug).unwrap();
                let mut matches = pages.iter().filter(|(_, m)| m.id == id).map(|(p, _)| p);
                let Some(fpath) = matches.next() else {
                    return Err(LocalStorageError::NoMatch(format!("id = {id}")));
                };
                let other_matches = matches.count();
                if other_matches > 0 {
                    return Err(LocalStorageError::TooManyMatches(other_matches, 1));
                }
                let (metadata, body) = self.load_content(fpath)?;
                Ok(StorageData::PageContent {
                    metadata,
                    body,
                    lang,
                })
            }

            StorageQueryMethod::ContentSlug(ref name) => {
                let path = self.get_content_path(&qry, Some(name), lang.as_ref(), Some("md"))?;
                let (metadata, body) = self.load_content(&path)?;
                Ok(StorageData::PageContent {
                    metadata,
                    body,
                    lang,
                })
            }

            StorageQueryMethod::RecentPages => {
                self.ensure_all_pages_loaded(&qry.storage_slug)?;
                let all_pages = self.all_pages.read();
                let mut results = all_pages
                    .get(&qry.storage_slug)
                    .unwrap()
                    .iter()
                    .filter(|(_, m)| !m.hidden)
                    .collect::<Vec<&(PathBuf, PageMetadata)>>();

                results.sort_by(|(_, a), (_, b)| a.compare_md(sort_key, b));
                if rev {
                    results.reverse();
                }

                let results = results
                    .into_iter()
                    .cloned()
                    .map(|(p, m)| m)
                    .take(if qry.limit == 0 {
                        usize::MAX
                    } else {
                        qry.limit
                    })
                    .collect();
                Ok(StorageData::RecentPages(results))
            }

            StorageQueryMethod::QueryTemplates => {
                let mut all_templates = HashMap::new();

                load_all_templates_from_dir(&self.template_root, vec![], &mut all_templates)?;
                Ok(StorageData::BaseTemplate(all_templates))
            }

            StorageQueryMethod::StaticFile(f) => {
                let fpath = PathBuf::from(f.trim_start_matches('/'));
                for inc in self.include_assets.iter() {
                    let try_path = inc.join(&fpath);
                    let try_path = try_path.absolutize().map_err(|e| {
                        LocalStorageError::BadRequest(format!("Absolutize {fpath:?}: {e:?}"))
                    })?;

                    if !try_path.starts_with(inc) {
                        log::error!("Possible directory traversal attack spotted");
                        log::error!("Got a request for static file {fpath:?}");
                        return Err(LocalStorageError::AttackSuspected(
                            "local-storage::static-file::directory-traversal".to_string(),
                        ));
                    }

                    let Some(fname) = try_path.file_name() else {
                        return Err(LocalStorageError::BadRequest("no file name".to_string()));
                    };
                    let Some(fname) = fname.to_str() else {
                        return Err(LocalStorageError::BadRequest(
                            "illegal characters in file name".to_string(),
                        ));
                    };

                    if try_path.is_file() {
                        return self.load_static_file(try_path.into_owned());
                    }

                    if let Some(ext) = try_path.extension() {
                        if ext == "css" {
                            return self.load_css(fname);
                        }
                    }
                }
                Err(LocalStorageError::DataNotFound(fpath))
            }

            StorageQueryMethod::GetSimilarPages((keys, val)) => {
                self.ensure_all_pages_loaded(&qry.storage_slug)?;
                let all_pages = self.all_pages.read();
                let pages = all_pages.get(&qry.storage_slug).unwrap();
                let mut matches = pages
                    .iter()
                    .filter(|(_, m)| {
                        !m.hidden && {
                            let valcmp = m.get_metadata(&keys);
                            match (valcmp, val.as_ref()) {
                                (Some(md), Some(val)) => compare_similar_md(md, val),
                                (Some(_), None) | (None, Some(_)) => false,
                                (None, None) => true,
                            }
                        }
                    })
                    .map(|(_, m)| m)
                    .collect::<Vec<&PageMetadata>>();

                matches.sort_by(|a, b| a.compare_md(sort_key, b));
                if rev {
                    matches.reverse();
                }

                let matches = matches
                    .into_iter()
                    .take(if qry.limit == 0 {
                        usize::MAX
                    } else {
                        qry.limit
                    })
                    .cloned()
                    .collect::<Vec<PageMetadata>>();
                Ok(StorageData::SimilarPages(matches))
            }

            StorageQueryMethod::QueryMetadata((keys, val), query) => {
                self.ensure_all_pages_loaded(&qry.storage_slug)?;
                let pages = self.all_pages.read().get(&qry.storage_slug).unwrap();
                let all_pages = self.all_pages.read();
                let pages = all_pages.get(&qry.storage_slug).unwrap();
                let matches = pages
                    .iter()
                    .filter(|(_, m)| m.get_metadata(&keys) == val.as_ref())
                    .filter_map(|(_, m)| m.get_metadata(&query).cloned())
                    .collect::<Vec<serde_json::Value>>();
                Ok(StorageData::QueryMetadata(matches))
            }
            StorageQueryMethod::QueryContext(ref name) => {
                let path = self.get_content_path(&qry, Some(name), lang.as_ref(), Some("toml"))?;
                let data = std::fs::read_to_string(&path)
                    .map_err(|e| LocalStorageError::LoadContext(format!("{path:?}: {e:?}")))?;
                let ctxt: toml::Value = toml::from_str(&data)
                    .map_err(|e| LocalStorageError::TomlDecode(format!("{path:?}: {e:?}")))?;
                Ok(StorageData::Context(ctxt))
            }
        }
    }

    pub fn select_lang(&self, qry: &StorageQuery) -> Result<Option<String>, LocalStorageError> {
        if let Some(ref lang) = qry.lang_pref {
            for l in lang.iter() {
                if self.supported_lang.contains(l) {
                    return Ok(Some(l.clone()));
                }
            }
            Ok(None)
        } else {
            Ok(None)
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
        Ok(storage)
    }

    // TODO Put a time argument to check for updates
    async fn has_changed(&self, qry: &StorageQuery) -> bool {
        // TODO    Find a way to know when posts are changed or not
        false
    }

    async fn query(&self, qry: StorageQuery) -> StorageData {
        match self.dispatch(qry) {
            Ok(data) => data,
            Err(e) => {
                log::error!("{e:?}");
                StorageData::Error(e)
            }
        }
    }
}

// TODO    Use this function once minify-js is fixed
pub fn minify_js(data: Vec<u8>) -> Vec<u8> {
    data
}

fn compare_similar_md(a: &serde_json::Value, b: &serde_json::Value) -> bool {
    if a.is_array() && b.is_array() {
        return a == b;
    }
    if a.is_array() {
        return a.as_array().unwrap().contains(b);
    }
    if b.is_array() {
        return b.as_array().unwrap().contains(a);
    }
    a == b
}

fn load_all_templates_from_dir(
    fpath: &PathBuf,
    parents: Vec<String>,
    hmap: &mut HashMap<String, String>,
) -> Result<(), LocalStorageError> {
    let entries = std::fs::read_dir(fpath).map_err(|e| {
        LocalStorageError::TemplateLoading(format!("Cannot list files in dir {fpath:?}: {e:?}"))
    })?;

    for path in entries {
        let path =
            path.map_err(|e| LocalStorageError::TemplateLoading(format!("Get path entry: {e:?}")))?;
        let path = path.path();
        if path.is_dir() {
            let Some(dname) = path.components().last().unwrap().as_os_str().to_str() else {
                log::warn!("Cannot load dir {path:?}: illegal dirname");
                continue;
            };
            let mut new_parents = parents.clone();
            new_parents.push(dname.to_string());
            load_all_templates_from_dir(&path, new_parents, hmap)?;
        } else {
            let Some(template_name) = path.file_name().unwrap().to_str() else {
                log::warn!("Cannot load file {path:?}: illegal filename");
                continue;
            };
            let mut template_path = parents.clone();
            template_path.push(template_name.to_string());
            let template_name = template_path.join("/");
            let content = std::fs::read_to_string(&path).map_err(|e| {
                LocalStorageError::TemplateLoading(format!(
                    "Loading template {path:?} error: {e:?}"
                ))
            })?;
            hmap.insert(template_name.to_string(), content);
        }
    }

    Ok(())
}
