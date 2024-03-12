mod data_extract;
mod request_handler;
mod static_files;
mod upload;

pub use upload::UploadEndpoint;

use actix_web::web::{self, ServiceConfig};
use request_handler::PageHandler;
use serde::{Deserialize, Serialize};

use crate::{config::Config, errors::Errcode, storage::{StorageQuery, StorageQueryMethod}};

use self::data_extract::RequestArgs;

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method", content = "args")]
#[serde(rename_all = "snake_case")]
pub enum ContentQueryMethod {
    // Get content slug from URL, with storage passed in parameter, has to be a str
    ContentSlug(String),

    // Get content ID from URL, with storage passed in parameter, has to be a number
    ContentId(String),

    #[default]
    FromSlug,
}

impl ContentQueryMethod {
    pub fn build_query(&self, storage: &String, args: &RequestArgs) -> Result<StorageQuery, Errcode> {
        let method = match self {
            ContentQueryMethod::ContentSlug(ref slug) => {
                let slug = args.get_query_slug(slug)?;
                StorageQueryMethod::ContentSlug(slug)
            },
            ContentQueryMethod::ContentId(ref slug) => {
                let id = args.get_query_id(slug)?;
                StorageQueryMethod::ContentNumId(id)
            },
            ContentQueryMethod::FromSlug => StorageQueryMethod::ContentNoId,
        };
        Ok(method.build_query(storage))
    }
}

pub fn configure(cfg: &Config, app: &mut ServiceConfig) {
    for (_, ptype) in cfg.page_type.iter() {
        app.route(
            ptype.route.as_str(),
            web::get().to(PageHandler::create(ptype)),
        );
    }
    upload::setup_routes(cfg, app);

    let static_endpoint = cfg.static_files_route
        .trim_end_matches("/")
        .to_string() + "{filename:.*}";
    app.route(
        &static_endpoint,
        web::get().to(static_files::StaticFilesRoute::init(&cfg)),
    );
}
