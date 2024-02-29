use serde::{Deserialize, Serialize};

use crate::page::PageType;
use crate::upload::UploadEndpoint;

#[derive(Serialize, Deserialize)]
pub struct Config {
    server_port: u16,
    pub default_lang: String,
    pub page_types: Vec<PageType>,
    pub upload_endpoints: Vec<UploadEndpoint>,
    pub add_contexts: Vec<String>,
}

impl Config {
    pub fn override_env_vars(&mut self) {
        // Override some configurations with environment variables
    }

    pub fn setup_app_base(&self) -> ServiceConfig {
        // Create Base application from server configurations
        todo!();
    }
}
