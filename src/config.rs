use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

// TODO    IMPORTANT     SECURITY     Check write permissions on data directory
//    If write permissions enabled on this directory, throw an error
//    except if dev feature is on.

#[cfg(feature = "githook")]
use crate::extensions::githook::GithookConfig;

#[cfg(feature = "webring")]
use crate::extensions::webring::WebringConfig;

#[cfg(feature = "humans-txt")]
use crate::extensions::humans::generate_humans_txt;

use crate::errors::Errcode;
use crate::loader::LoadingLimits;
use crate::Args;

#[cfg(feature = "local_storage")]
type StorageConfig = crate::loader::storage::local_storage::LocalStorageConfig;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Configuration {
    pub scss_dir: PathBuf,
    pub scripts_dir: PathBuf,
    pub templates_dir: PathBuf,
    pub site_config_file: PathBuf,

    pub add_assets: Vec<PathBuf>,

    pub server_port: u16,
    pub req_limit_per_sec: usize,
    pub code_theme: String,
    pub limits: LoadingLimits,

    #[cfg(feature = "local_storage")]
    pub posts_registry: PathBuf,
    #[cfg(feature = "local_storage")]
    pub posts_dir: PathBuf,

    #[serde(default)]
    pub templates: HashMap<String, String>,
    #[serde(default)]
    pub scss: HashMap<String, Vec<String>>,

    #[serde(skip)]
    pub storage_cfg: StorageConfig,
    #[serde(skip)]
    pub assets_dir: PathBuf,
    #[serde(skip)]
    pub site_config: SiteConfig,
    #[serde(skip)]
    pub data_dir: PathBuf,

    // Extensions
    #[cfg(feature = "add-endpoint")]
    pub add_endpoints: HashMap<String, PathBuf>,

    #[serde(skip)]
    #[cfg(feature = "save-data")]
    pub save_data_dir: PathBuf,
}

impl From<Args> for Configuration {
    fn from(args: Args) -> Self {
        let configf = args.data_dir.join("config.toml");
        let config: Configuration = toml::from_str(
            std::fs::read_to_string(configf)
                .expect("Unable to read config")
                .as_str(),
        )
        .expect("Unable to deserialize config file");
        Configuration {
            // Skipped
            data_dir: args.data_dir.clone(),
            storage_cfg: StorageConfig::init(&args.data_dir, &config),
            site_config: SiteConfig::init(&args.data_dir, &config),
            assets_dir: args.assets_dir,

            // Directories relative to data root
            scss_dir: args.data_dir.join(config.scss_dir),
            scripts_dir: args.data_dir.join(config.scripts_dir),
            templates_dir: args.data_dir.join(config.templates_dir),
            site_config_file: args.data_dir.join(config.site_config_file),
            posts_registry: args.data_dir.join(config.posts_registry),
            posts_dir: args.data_dir.join(config.posts_dir),
            add_assets: config
                .add_assets
                .into_iter()
                .map(|p| args.data_dir.join(p))
                .collect(),

            // Extensions
            #[cfg(feature = "save-data")]
            save_data_dir: args.save_data_dir,

            // Static configs
            ..config
        }
    }
}

impl Configuration {
    pub async fn validate(&self) -> Result<(), Errcode> {
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
        for asset in self.add_assets.iter() {
            if !asset.exists() {
                return Err(Errcode::PathDoesntExist("add asset", asset.clone()));
            }
        }
        tokio::fs::create_dir_all(&self.assets_dir).await?;
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

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct SiteConfig {
    pub name: String,
    pub base_url: String,
    pub favicon: PathBuf,
    pub og_image: Option<PathBuf>,
    pub author_name: String,
    pub author_email: String,
    pub description: String,
    welcome_message: String,
    pub copyrights: String,

    pub social: HashMap<String, String>,

    pub blog_engine_src: Option<String>,
    pub blog_src: Option<String>,

    // Extensions
    #[cfg(feature = "githook")]
    pub webhook_update: GithookConfig,

    #[cfg(feature = "webring")]
    pub webring: WebringConfig,

    #[cfg(feature = "humans-txt")]
    #[serde(default)]
    pub humans_txt: String,

    #[cfg(feature = "add-endpoint")]
    pub additionnal_context: HashMap<String, PathBuf>,

    #[cfg(feature = "save-data")]
    pub allowed_savedata_tokens: Vec<String>,
}

impl SiteConfig {
    #[allow(unused_mut)]
    pub fn init(root: &Path, config: &Configuration) -> SiteConfig {
        let sitef = root.join(&config.site_config_file);
        let mut site_config: SiteConfig = toml::from_str(
            std::fs::read_to_string(sitef)
                .expect("Unable to read site config file")
                .as_str(),
        )
        .expect("Error while decoding data from site config file");

        #[cfg(feature = "humans-txt")]
        generate_humans_txt(&mut site_config);

        SiteConfig {
            favicon: root.join(site_config.favicon),
            ..site_config
        }
    }
}
