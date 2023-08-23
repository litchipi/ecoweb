use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{cache::CacheConfig, errors::Errcode, loader::LoadingLimits, Args};

#[cfg(feature = "local_storage")]
type StorageConfig = crate::loader::storage::local_storage::LocalStorageConfig;

#[derive(Clone, Serialize, Deserialize)]
pub struct Configuration {
    #[serde(skip)]
    pub scss_dir: PathBuf,
    #[serde(skip)]
    pub scripts_dir: PathBuf,
    #[serde(skip)]
    pub templates_dir: PathBuf,
    #[serde(skip)]
    pub site_config_file: PathBuf,
    #[serde(skip)]
    pub favicon: PathBuf,
    #[serde(skip)]
    pub add_assets: Vec<PathBuf>,
    #[serde(skip)]
    pub storage_cfg: StorageConfig,
    #[serde(skip)]
    pub assets_dir: PathBuf,

    pub server_port: u16,
    pub cache: CacheConfig,
    pub req_limit_per_sec: usize,
    pub code_theme: String,
    pub limits: LoadingLimits,

    #[serde(default)]
    pub templates: HashMap<String, String>,
    #[serde(default)]
    pub scss: HashMap<String, Vec<String>>,
}

impl From<Args> for Configuration {
    fn from(args: Args) -> Self {
        let config: Configuration = toml::from_str(
            std::fs::read_to_string(&args.config_file)
                .unwrap_or_else(|_| panic!("Config file {:?} not found", args.config_file))
                .as_str(),
        )
        .expect("Unable to deserialize config file");
        Configuration {
            storage_cfg: StorageConfig::from(&args),
            site_config_file: args.site_config_file,
            scss_dir: args.scss_dir,
            scripts_dir: args.scripts_dir,
            templates_dir: args.templates_dir,
            favicon: args.favicon,
            add_assets: args.add_assets,
            assets_dir: args.assets_dir,
            ..config
        }
    }
}

impl Configuration {
    pub fn validate(&self) -> Result<(), Errcode> {
        self.storage_cfg.validate()?;
        if !self.site_config_file.exists() {
            return Err(Errcode::PathDoesntExist(
                "site config",
                self.site_config_file.clone(),
            ));
        }
        if !self.templates_dir.exists() {
            return Err(Errcode::PathDoesntExist(
                "templates",
                self.templates_dir.clone(),
            ));
        }
        if !self.scss_dir.exists() {
            return Err(Errcode::PathDoesntExist("scss", self.scss_dir.clone()));
        }
        if !self.scripts_dir.exists() {
            return Err(Errcode::PathDoesntExist(
                "scripts",
                self.scripts_dir.clone(),
            ));
        }
        if !self.favicon.exists() {
            return Err(Errcode::PathDoesntExist("favicon", self.favicon.clone()));
        }
        for asset in self.add_assets.iter() {
            if !asset.exists() {
                return Err(Errcode::PathDoesntExist("add asset", asset.clone()));
            }
        }
        std::fs::create_dir_all(&self.assets_dir)?;
        Ok(())
    }

    pub fn init_logging(&self) {
        let mut builder = env_logger::Builder::new();
        builder.filter_level(log::LevelFilter::Debug);
        builder.parse_env("RUST_LOG");
        builder.init();
        log::debug!("Logging started");
    }

    pub fn get_grass_options(&self) -> grass::Options {
        grass::Options::default()
    }
}
