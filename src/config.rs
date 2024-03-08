use actix_web::http::header;
use actix_web::middleware::DefaultHeaders;
use serde::{Deserialize, Serialize};

use crate::page::PageType;
use crate::routes::UploadEndpoint;

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    pub server_port: u16,
    pub default_lang: String,
    pub page_types: Vec<PageType>,
    pub upload_endpoints: Vec<UploadEndpoint>,
}

impl Config {
    // TODO    Get from args, or environment variables
    pub fn init() -> Config {
        Config {
            server_port: 8083,
            default_lang: "fr".to_string(),
            page_types: vec![ PageType::test() ],
            upload_endpoints: vec![],
        }
    }

    pub fn setup_logging(&self) {
        // TODO    Setup logging
    }

    pub fn get_default_headers(&self) -> DefaultHeaders {
        DefaultHeaders::new()
            .add((header::CONTENT_TYPE, "text/html; charset=UTF-8"))
            .add((header::X_CONTENT_TYPE_OPTIONS, "nosniff"))
            .add((header::X_FRAME_OPTIONS, "DENY"))
            .add((
                header::PERMISSIONS_POLICY,
                "geolocation=(), camera=(), microphone=()",
            ))
            // .add((header::CACHE_CONTROL, format!("max-age={MAX_AGE}")))
            // .add((header::AGE, "0")),
    }
}
