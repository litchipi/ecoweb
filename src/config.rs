use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{errors::Errcode, loader::LoadingLimits, Args, endpoints::githook::WebhookConfig};

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
}

impl From<Args> for Configuration {
    fn from(args: Args) -> Self {
        let configf = args.data_dir.join("config.toml");
        let config: Configuration = toml::from_str(
            std::fs::read_to_string(&configf).expect("Unable to read config")
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
            add_assets: config.add_assets.into_iter().map(|p| args.data_dir.join(p)).collect(),

            // Static configs
            ..config
        }
    }
}

impl Configuration {
    pub fn validate(&self) -> Result<(), Errcode> {
        println!("{:?}", self.storage_cfg);
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
    pub webring: WebringContext,
    pub webhook_update: WebhookConfig,

    blog_engine_src: Option<String>,
    pub blog_src: Option<String>,

    #[serde(default)]
    pub humans_txt: String,
}

impl SiteConfig {
    pub fn init(root: &PathBuf, config: &Configuration) -> SiteConfig {
        let sitef = root.join(&config.site_config_file);
        let mut site_config : SiteConfig = toml::from_str(
                std::fs::read_to_string(sitef)
                    .expect("Unable to read site config file").as_str()
            ).expect("Error while decoding data from site config file");
        site_config.generate_humans_txt();
        SiteConfig {
            favicon: root.join(site_config.favicon),
            og_image: site_config.og_image.map(|i| root.join(i)),
            ..site_config
        }
    }

    fn generate_humans_txt(&mut self) {
        self.humans_txt = String::new();
        self.humans_txt += "/* TEAM */\n";
        self.humans_txt += format!("Author: {}\n", self.author_name).as_str();
        for (sitename, social) in self.social.iter() {
            if sitename == "email" {
                let address = self.author_email.replace('@', " [at] ");
                self.humans_txt += format!("Email: {}\n", address).as_str();
            } else {
                let mut s = sitename.chars();
                let sitename_cap = match s.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + s.as_str(),
                };
                self.humans_txt += format!("{}: {}\n", sitename_cap, social).as_str();
            }
        }
        if let Some(ref blog_engine) = self.blog_engine_src {
            self.humans_txt += format!("\nSoftware sources: {}\n", blog_engine).as_str();
        }
        if let Some(ref blog_src) = self.blog_src {
            self.humans_txt += format!("Content sources: {}\n", blog_src).as_str();
        }
        self.humans_txt += "\nLanguage: English\n";
    }

    pub fn to_rss_feed(&self, xml: &mut String) {
        *xml += format!("<title>{}</title>", self.name).as_str();
        *xml += format!("<link>{}</link>", self.base_url).as_str();
        *xml += format!("<description>{}</description>", self.description).as_str();
        *xml += format!(
            "<managingEditor>{} ({})</managingEditor>",
            self.author_email, self.author_name,
        )
        .as_str();
        *xml += format!(
            "<webMaster>{} ({})</webMaster>",
            self.author_email, self.author_name
        )
        .as_str();
        *xml += format!("<copyright>{}</copyright>", self.copyrights).as_str();
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct WebringContext {
    name: String,
    next: String,
    previous: String,
}
