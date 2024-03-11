mod data_extract;
mod request_handler;
mod upload;

pub use upload::UploadEndpoint;

use actix_web::web::{self, ServiceConfig};
use request_handler::PageHandler;
use serde::{Deserialize, Serialize};

use crate::config::Config;

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method", content = "args")]
#[serde(rename_all = "snake_case")] 
pub enum ContentQueryMethod {
    // Get content ID from URL, with slug passed in parameter, has to be a number
    ContentId(String),

    #[default]
    FromSlug,
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
