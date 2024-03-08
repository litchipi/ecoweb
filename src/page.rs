use std::collections::HashMap;

use actix_web::HttpRequest;
use serde::{Deserialize, Serialize};

use crate::storage::{ContextQuery, StorageQuery, StorageSlug};
use crate::dispatch::UrlBuildMethod;
use crate::render::TemplateSlug;


#[derive(Serialize, Deserialize, Clone)]
pub struct PageType {
    pub route: String,
    pub lang_detect: bool,
    pub add_context: HashMap<String, ContextQuery>,
    pub default_template: TemplateSlug,
    url_build_method: UrlBuildMethod,
    storage: StorageSlug,
}

impl PageType {
    pub fn build_query(&self, req: &HttpRequest) -> StorageQuery {
        StorageQuery::content_from(&self.url_build_method, req)
    }

    pub fn build_query_with_lang(&self, req: &HttpRequest, langs: Vec<String>) -> StorageQuery {
        let mut qry = StorageQuery::content_from(&self.url_build_method, req);
        qry.lang_pref = Some(langs);
        qry
    }
}
