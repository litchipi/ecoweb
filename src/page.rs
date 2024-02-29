use serde::{Deserialize, Serialize};

use crate::storage::{ContextQuery, StorageQuery, StorageSlug};
use crate::dispatch::UrlBuildMethod;
use crate::render::TemplateSlug;


#[derive(Serialize, Deserialize, Clone)]
pub struct PageType {
    pub add_context: Vec<ContextQuery>,
    pub lang_detect: bool,
    pub route: String,
    pub default_template: TemplateSlug,
    url_build_method: UrlBuildMethod,
    storage: StorageSlug,
}

impl PageType {
    pub fn build_query(&self, req: &RequestParams) -> StorageQuery {
        StorageQuery::content_from(&self.url_build_method, req)
    }

    pub fn build_query_with_lang(&self, req: &RequestParams, langs: Vec<String>) -> StorageQuery {
        let mut qry = StorageQuery::content_from(&self.url_build_method, req);
        qry.lang_pref = Some(langs);
        qry
    }
}
