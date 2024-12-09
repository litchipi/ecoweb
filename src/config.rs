use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use actix_web::http::header;
use actix_web::middleware::DefaultHeaders;
use clap::Parser;
use serde::{Deserialize, Serialize};
use tera::Context;

use crate::errors::Errcode;
use crate::page::PageType;
use crate::routes::UploadEndpoint;
use crate::storage::{ContextQuery, Storage};

#[derive(Parser)]
struct Arguments {
    #[arg(short, long)]
    config_file: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip)]
    pub root: PathBuf, // Derived from the config file path

    pub server_port: u16,
    pub default_lang: String,
    pub static_files_route: String,

    pub notification_template: String,

    #[serde(default)]
    pub plain_context: HashMap<String, serde_json::Value>,

    #[serde(default)]
    pub add_context: HashMap<String, ContextQuery>,

    page_config: PathBuf,

    #[serde(default)]
    pub page_type: HashMap<String, PageType>,

    #[serde(default)]
    pub upload_endpoints: HashMap<String, UploadEndpoint>,

    #[serde(default)]
    pub redirections: HashMap<String, String>,

    #[cfg(feature = "storage-local")]
    pub local_storage: crate::storage::backend::local::LocalStorage,
}

impl Config {
    pub fn load() -> Result<Config, Errcode> {
        let args = Arguments::parse();
        let config_str = std::fs::read_to_string(&args.config_file)
            .map_err(|e| Errcode::ConfigFileRead(Arc::new(e)))?;
        let mut config: Config =
            toml::from_str(&config_str).map_err(|e| Errcode::TomlDecode("config file", e))?;
        config.root = args.config_file.parent().unwrap().to_path_buf();

        let page_def_str = std::fs::read_to_string(config.root.join(&config.page_config))
            .map_err(|e| Errcode::ConfigFileRead(Arc::new(e)))?;
        let mut page_def: HashMap<String, PageType> =
            toml::from_str(&page_def_str).map_err(|e| Errcode::TomlDecode("page def", e))?;

        for (slug, ptype) in page_def.iter_mut() {
            if ptype.storage.is_empty() {
                ptype.storage = slug.clone();
            }
        }
        config.page_type = page_def;

        Ok(config)
    }

    pub fn setup_logging(&self) {
        let mut builder = env_logger::Builder::new();
        builder.filter_level(log::LevelFilter::Debug);
        builder.parse_env("RUST_LOG");
        builder.init();
        log::debug!("Logging started");
    }

    pub fn get_default_headers(&self) -> DefaultHeaders {
        DefaultHeaders::new()
            .add((header::CONTENT_TYPE, "text/html; charset=UTF-8"))
            .add((header::X_CONTENT_TYPE_OPTIONS, "nosniff"))
            .add((header::X_FRAME_OPTIONS, "DENY"))
            .add((
                header::PERMISSIONS_POLICY,
                "geolocation=(), camera=(), microphone=()",
            ))
        // .add((header::CACHE_CONTROL, format!("max-age={MAX_AGE}")))
        // .add((header::AGE, "0")),
    }

    pub async fn base_templating_context(&self, storage: &Storage) -> Result<Context, Errcode> {
        let mut ctxt = Context::new();
        ctxt.insert("default_lang", &self.default_lang);
        for (slug, data) in self.plain_context.iter() {
            ctxt.insert(slug, data);
        }

        for (slug, qry) in self.add_context.iter() {
            if let ContextQuery::Plain(d) = qry {
                ctxt.insert(slug, d);
            }
            let sq = qry.independant_query()?.unwrap();
            let val = storage.query(sq).await;
            qry.insert_data(slug, &mut ctxt, val)?;
        }
        Ok(ctxt)
    }
}
