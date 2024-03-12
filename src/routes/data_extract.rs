use actix_web::dev::{Path, Payload, Url};
use actix_web::web::Data;
use actix_web::{FromRequest, HttpRequest};
use tera::Context;

use crate::errors::Errcode;
use crate::render::Render;
use crate::storage::Storage;

#[derive(Clone)]
pub struct RequestArgs {
    pub uri: String,
    pub lang: Option<Vec<String>>,
    pub storage: Data<Storage>,
    pub render: Data<Render>,
    pub base_context: Data<Context>,
    pub match_infos: Path<Url>,
}

impl FromRequest for RequestArgs {
    type Error = actix_web::Error;
    type Future = std::future::Ready<Result<RequestArgs, Self::Error>>;

    // Function called everytime we have a request to handle
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        std::future::ready(Ok(RequestArgs {
            uri: req.uri().to_string(),
            lang: get_lang(req),
            storage: get_from_req(req),
            render: get_from_req(req),
            base_context: get_from_req(req),
            match_infos: req.match_info().clone(),
        }))
    }
}

impl RequestArgs {
    pub fn get_query_id(&self, slug: &String) -> Result<u64, Errcode> {
        if let Some(id) = self.match_infos.get(slug) {
            Ok(id
                .parse::<u64>()
                .map_err(|e| Errcode::ContentIdParsing(e))?)
        } else {
            Err(Errcode::ParameterNotInUrl)
        }
    }
}

// TODO    Get lang from request
fn get_lang(req: &HttpRequest) -> Option<Vec<String>> {
    // Get langs from headers or GET params
    None
}

fn get_from_req<T: 'static>(req: &HttpRequest) -> Data<T> {
    let what = std::any::type_name::<T>();
    let data = req
        .app_data::<Data<T>>()
        .expect(format!("Unable to get {what} from app_data").as_str());
    data.clone()
}

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
