use std::{future::Future, pin::Pin};

use actix_web::{body::BoxBody, http::header, web::Data, Handler, HttpResponse, HttpResponseBuilder};

use crate::{config::Config, storage::{Storage, StorageQuery}};

use super::data_extract::RequestArgs;

#[derive(Clone)]
pub struct StaticFilesRoute {
    prefix: String,
}

impl StaticFilesRoute {
    pub fn init(cfg: &Config) -> StaticFilesRoute {
        StaticFilesRoute {
            prefix: cfg.static_files_route.clone(),
        }
    }

    pub async fn serve_file(fname: Option<String>, storage: Data<Storage>) -> HttpResponse<BoxBody> {
        let Some(fname) = fname else {
            return HttpResponse::InternalServerError().body("URI doesn't have prefix");
        };

        let mime = mime_guess::from_path(&fname).first_or_octet_stream();

        let qry = StorageQuery::static_file(fname);
        match storage.query(qry).await.static_file() {
            Ok(data) => HttpResponse::Ok()
                .insert_header(header::ContentType(mime))
                .body(data),
            Err(e) => {
                let msg = format!("{e:?}");
                let mut err : HttpResponseBuilder = e.into();
                err.body(msg)
            },
        }
    }
}

impl Handler<RequestArgs> for StaticFilesRoute {
    type Output = HttpResponse<BoxBody>;

    type Future = Pin<Box<dyn Future<Output = Self::Output>>>;

    fn call(&self, args: RequestArgs) -> Self::Future {
        let fname = args.uri.strip_prefix(&self.prefix).map(|s| s.to_string());
        Box::pin(Self::serve_file(fname, args.storage))
    }
}
