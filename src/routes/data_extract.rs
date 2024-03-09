use actix_web::dev::Payload;
use actix_web::web::Data;
use actix_web::{FromRequest, HttpRequest};
use std::future::Ready;

use crate::render::Render;
use crate::storage::Storage;

#[derive(Clone)]
pub struct RequestArgs {
    pub lang: Option<Vec<String>>,
    pub storage: Data<Storage>,
    pub render: Data<Render>,
    // TODO    Add base context
}

async fn test() {}

impl FromRequest for RequestArgs {
    type Error = actix_web::Error;
    type Future = Ready<Result<RequestArgs, Self::Error>>;

    // TODO    Handle error cases
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let storage: Option<&Data<Storage>> = req.app_data();
        let render: Option<&Data<Render>> = req.app_data();

        // HttpRequest HTTP/1.1 GET:/toto
        //   headers:
        //     "accept": "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8"
        //     "accept-encoding": "gzip, deflate"
        //     "cookie": "*redacted*"
        //     "host": "0.0.0.0:8083"
        //     "user-agent": "Mozilla/5.0 (X11; Linux x86_64; rv:123.0) Gecko/20100101 Firefox/123.0"
        //     "connection": "keep-alive"
        //     "accept-language": "fr,fr-FR;q=0.8,en-US;q=0.5,en;q=0.3"
        //     "upgrade-insecure-requests": "1"

        // TODO    Find a way to call async functions from here
        // test().await

        let res = Ok(RequestArgs {
            lang: get_lang(req),
            storage: storage.unwrap().clone(),
            render: render.unwrap().clone(),
        });

        std::future::ready(res)
    }
}

// TODO    Get lang from request
fn get_lang(req: &HttpRequest) -> Option<Vec<String>> {
    // Get langs from headers or GET params
    None
}
