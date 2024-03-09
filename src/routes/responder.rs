use actix_web::{body::BoxBody, HttpRequest, HttpResponse, Responder};
use tera::Context;

use crate::errors::Errcode;
use crate::page::PageType;
use crate::storage::StorageQuery;

use super::data_extract::RequestArgs;

pub struct RequestReponder {
    args: RequestArgs,
    ctxt: Context,
    storage_slug: String,
}

impl RequestReponder {
    pub fn create(ptype: &PageType, args: RequestArgs) -> RequestReponder {
        RequestReponder {
            ctxt: Context::new(), // TODO    Clone from args
            storage_slug: ptype.storage.clone(),
            args,
        }
    }

    pub fn handle_request(&mut self, req: &HttpRequest) -> Result<String, Errcode> {
        // TODO    Query the storage for data
        let qry = StorageQuery::recent_pages(&self.storage_slug, 5);
        let post = self.args.storage.query(&qry);
        // TODO    Add to context
        // TODO    Register template if not already cached
        // TODO    Render post based on data
        Ok(format!("{post:?}"))
    }

    pub fn with_headers(&self, reply: HttpResponse<BoxBody>) -> HttpResponse<BoxBody> {
        // TODO    Additionnal headers here
        reply
    }
}

impl Responder for RequestReponder {
    type Body = BoxBody;

    fn respond_to(mut self, req: &HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        // TODO    Get data from storage
        // TODO    Populate context
        // TODO    Render template
        // TODO    Handle any error case
        let res = self.handle_request(req);
        let reply = match res {
            Ok(body) => HttpResponse::Ok().body(body),
            // TODO    Handle error cases
            Err(e) => HttpResponse::InternalServerError().body(format!("{e:?}")),
        };
        self.with_headers(reply)
    }
}
