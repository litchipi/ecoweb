use std::future::Ready;
use actix_web::{dev::Payload, web::Data, FromRequest, HttpRequest};

use crate::render::Render;
use crate::storage::Storage;

#[derive(Clone)]
pub struct RequestArgs {
    pub lang: Option<Vec<String>>,
    pub storage: Data<Storage>,
    pub render: Data<Render>,
    // TODO    Add base context
}

impl FromRequest for RequestArgs {
    type Error = actix_web::Error;
    type Future = Ready<Result<RequestArgs, Self::Error>>;

    // TODO    Handle error cases
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let storage : Option<&Data<Storage>> = req.app_data();
        let render : Option<&Data<Render>> = req.app_data();

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
