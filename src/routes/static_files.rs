use std::future::Future;
use std::pin::Pin;

use actix_web::body::BoxBody;
use actix_web::http::header;
use actix_web::web::Data;
use actix_web::{Handler, HttpResponse, HttpResponseBuilder};

use crate::config::Config;
use crate::page::default_cache_max_age;
use crate::storage::{Storage, StorageQuery};

use super::data_extract::RequestArgs;

#[derive(Clone)]
pub struct StaticFilesRoute;
impl StaticFilesRoute {
    pub fn init(cfg: &Config) -> StaticFilesRoute {
        StaticFilesRoute
    }

    pub async fn serve_file(fname: String, storage: Data<Storage>) -> HttpResponse<BoxBody> {
        let mime = mime_guess::from_path(&fname).first_or_octet_stream();

        let qry = StorageQuery::static_file(fname);
        match storage.query(qry).await.static_file() {
            Ok(data) => HttpResponse::Ok()
                .insert_header(header::ContentType(mime))
                .insert_header(header::CacheControl(vec![ header::CacheDirective::MaxAge(default_cache_max_age()) ]))
                .body(data),
            Err(e) => {
                log::warn!("Unable to get file: {e:?}");
                let msg = format!("{e:?}");
                let mut err: HttpResponseBuilder = e.into();
                err.body(msg)
            }
        }
    }
}

impl Handler<RequestArgs> for StaticFilesRoute {
    type Output = HttpResponse<BoxBody>;

    type Future = Pin<Box<dyn Future<Output = Self::Output>>>;

    fn call(&self, args: RequestArgs) -> Self::Future {
        // TODO    Add caching headers to request
        let fname = args.match_infos.get("filename").unwrap();
        Box::pin(Self::serve_file(fname.to_string(), args.storage))
    }
}
