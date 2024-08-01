use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use actix_web::body::BoxBody;
use actix_web::dev::Payload;
use actix_web::web::{Data, Form};
use actix_web::{FromRequest, Handler, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use tera::Context;

use crate::config::Config;
use crate::errors::Errcode;
use crate::render::Render;
use crate::routes::data_extract::get_lang;
use crate::storage::Storage;

pub type FormData = HashMap<String, String>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostFormNotification {
    title: String,
    message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormAction {
    pub endpoint: String,
    pub method: FormActionMethod,
    notification: HashMap<String, PostFormNotification>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", content = "args")]
pub enum FormActionMethod {
    NoAction,
    SendOverEmail(String),
}

impl FormAction {
    pub fn create_handler(&self) -> FormRequestHandler {
        FormRequestHandler(self.clone())
    }

    pub async fn action(&self, req: &FormReq) -> Result<(), Errcode> {
        match self.method {
            FormActionMethod::NoAction => Ok(()),
            FormActionMethod::SendOverEmail(ref subject) => {
                let serialized = bincode::serialize(&req.data)?;
                req.config.mail.as_ref().unwrap().send_data(subject, &serialized)?;
                Ok(())
            }
        }
    }
}

pub struct FormReq {
    config: Data<Config>,
    render: Data<Render>,
    storage: Data<Storage>,
    ctxt: Context,
    data: Form<FormData>,
    lang: Option<Vec<String>>,
}

impl FromRequest for FormReq {
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<FormReq, Self::Error>>>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let config = req
            .app_data::<Data<Config>>()
            .unwrap_or_else(|| panic!("{}", "Unable to get config from app_data".to_string()))
            .clone();
        let render = req
            .app_data::<Data<Render>>()
            .unwrap_or_else(|| panic!("{}", "Unable to get render from app_data".to_string()))
            .clone();
        let storage = req
            .app_data::<Data<Storage>>()
            .unwrap_or_else(|| panic!("{}", "Unable to get storage from app_data".to_string()))
            .clone();
        let ctxt = req
            .app_data::<Data<Context>>()
            .unwrap_or_else(|| panic!("{}", "Unable to get tera context from app_data".to_string()))
            .clone();
        let lang = get_lang(req);
        let mut ctxt = ctxt.get_ref().clone();
        ctxt.insert("pref_langs", &lang);

        let req = req.clone();
        let mut payload = payload.take();
        Box::pin(async move {
            let form_data = Form::from_request(&req, &mut payload).await;
            log::debug!("{form_data:?}");
            match form_data {
                Ok(data) => Ok(FormReq {
                    config,
                    storage,
                    ctxt,
                    render,
                    data,
                    lang,
                }),
                Err(e) => Err(e),
            }
        })
    }
}

#[derive(Clone)]
pub struct FormRequestHandler(FormAction);

impl Handler<FormReq> for FormRequestHandler {
    type Output = HttpResponse<BoxBody>;

    type Future = Pin<Box<dyn Future<Output = Self::Output>>>;

    fn call(&self, req: FormReq) -> Self::Future {
        Box::pin(Self::handle(self.0.clone(), req))
    }
}

impl FormRequestHandler {
    async fn handle(action: FormAction, req: FormReq) -> HttpResponse<BoxBody> {
        let res = action.action(&req).await;
        match res {
            Ok(_) => {
                let lang = req
                    .lang
                    .and_then(|langs| {
                        langs
                            .iter()
                            .filter(|l| action.notification.contains_key(*l))
                            .nth(0)
                            .cloned()
                    })
                    .unwrap_or(req.config.default_lang.clone());
                let notification = action.notification.get(&lang).unwrap();
                match req
                    .render
                    .render_notification(
                        notification.title.clone(),
                        notification.message.clone(),
                        req.ctxt.clone(), // I don't like this
                    )
                    .await
                {
                    Ok(body) => HttpResponse::Ok().body(body),
                    Err(e) => {
                        e.build_http_response(&req.render, req.ctxt.clone())
                            .await
                    }
                }
            }
            Err(e) => {
                e.build_http_response(&req.render, req.ctxt).await
            }
        }
    }
}
