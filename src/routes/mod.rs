pub mod data_extract;
mod request_handler;
mod static_files;
mod upload;

pub use upload::UploadEndpoint;

use actix_web::web::{self, ServiceConfig};
use request_handler::PageHandler;
use serde::{Deserialize, Serialize};

use crate::{
    config::Config,
    errors::Errcode,
    storage::{StorageQuery, StorageQueryMethod},
};

pub use self::data_extract::RequestArgs;

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method", content = "args")]
#[serde(rename_all = "snake_case")]
pub enum ContentQueryMethod {
    // No content to get, but populate context
    #[default]
    EmptyContent,

    // Get content slug from URL, with storage passed in parameter, has to be a str
    ContentSlug(String),

    // Get content ID from URL, with storage passed in parameter, has to be a number
    ContentId(String),

    FromName(String),
}

impl ContentQueryMethod {
    pub fn build_query(
        &self,
        storage: &String,
        args: &RequestArgs,
    ) -> Result<StorageQuery, Errcode> {
        let method = match self {
            ContentQueryMethod::EmptyContent => StorageQueryMethod::NoOp,
            ContentQueryMethod::ContentSlug(ref slug) => {
                let slug = args.get_query_slug(slug)?;
                StorageQueryMethod::ContentSlug(slug)
            }
            ContentQueryMethod::ContentId(ref slug) => {
                let id = args.get_query_id(slug)?;
                StorageQueryMethod::ContentNumId(id)
            }
            ContentQueryMethod::FromName(name) => StorageQueryMethod::ContentFromName(name.clone()),
        };
        Ok(method.build_query(storage))
    }
}

pub fn configure(cfg: &Config, app: &mut ServiceConfig) {
    for (from, to) in cfg.redirections.iter() {
        app.service(web::redirect(from.clone(), to.clone()));
    }

    for (_, ptype) in cfg.page_type.iter() {
        app.route(
            ptype.route.as_str(),
            web::get().to(PageHandler::create(ptype, &cfg.default_lang)),
        );
    }
    upload::setup_routes(cfg, app);

    let static_endpoint =
        cfg.static_files_route.trim_end_matches('/').to_string() + "{filename:.*}";
    app.route(
        &static_endpoint,
        web::get().to(static_files::StaticFilesRoute::init(cfg)),
    );
}
