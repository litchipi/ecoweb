use actix_web::Handler;
use std::future::Ready;

use super::data_extract::RequestArgs;
use super::responder::RequestReponder;
use crate::page::PageType;

#[derive(Clone)]
pub struct PageHandler {
    ptype: PageType,
}

impl Handler<RequestArgs> for PageHandler {
    type Output = RequestReponder;
    type Future = Ready<Self::Output>;

    fn call(&self, args: RequestArgs) -> Self::Future {
        std::future::ready(RequestReponder::create(&self.ptype, args))
    }
}

impl PageHandler {
    pub fn create(ptype: &PageType) -> PageHandler {
        PageHandler {
            ptype: ptype.clone(),
        }
    }
}
