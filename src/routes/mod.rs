mod data_extract;
mod request_handler;
mod upload;

pub use upload::UploadEndpoint;

use actix_web::web::{self, ServiceConfig};
use request_handler::PageHandler;
use serde::{Deserialize, Serialize};

use crate::config::Config;

// TODO IMPORTANT    Clean way to create a StorageQuery from a URL and a PageType
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")] 
pub enum UrlBuildMethod {
    ContentId,
}

pub fn configure(cfg: &Config, app: &mut ServiceConfig) {
    for (_, ptype) in cfg.page_type.iter() {
        app.route(
            ptype.route.as_str(),
            web::get().to(PageHandler::create(ptype)),
        );
    }
    upload::setup_routes(cfg, app);
    // TODO    Setup static files endpoint
}
