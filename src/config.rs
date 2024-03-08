use actix_web::web::ServiceConfig;
use serde::{Deserialize, Serialize};

use crate::page::PageType;
use crate::upload::UploadEndpoint;

#[derive(Serialize, Deserialize)]
pub struct Config {
    server_port: u16,
    pub default_lang: String,
    pub page_types: Vec<PageType>,
    pub upload_endpoints: Vec<UploadEndpoint>,
}

impl Config {
    pub fn init() -> Config {
        Config {
            server_port: 8083,
            default_lang: "fr".to_string(),
            page_types: vec![],
            upload_endpoints: vec![],
        }
    }
    pub fn override_env_vars(&mut self) {
        // Override some configurations with environment variables
    }

    pub fn setup_app_base(&self) -> ServiceConfig {
        // Create Base application from server configurations
        todo!();
    }
}
