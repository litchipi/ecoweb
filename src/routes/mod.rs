
mod request_handler;
mod data_extract;
mod responder;
mod upload;

pub use upload::UploadEndpoint;

use actix_web::web::{self, ServiceConfig};
use request_handler::PageHandler;
use serde::{Deserialize, Serialize};

use crate::config::Config;

#[derive(Serialize, Deserialize, Clone)]
pub enum UrlBuildMethod {
    ContentId,
    FromMetadata(String),  // Metadata key
}

pub fn configure(cfg: &Config, app: &mut ServiceConfig) {
    for ptype in cfg.page_types.iter() {
        app.route(
            ptype.route.as_str(),
            web::get().to(PageHandler::create(ptype)),
        );
    }
    upload::setup_routes(cfg, app);
    // TODO    Setup static files endpoint
}
