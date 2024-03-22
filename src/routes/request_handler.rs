use actix_web::body::BoxBody;
use actix_web::web::Data;
use actix_web::{Handler, HttpResponse, HttpResponseBuilder};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use tera::Context;

use super::data_extract::RequestArgs;
use crate::errors::Errcode;
use crate::page::{PageMetadata, PageType};
use crate::render::Render;
use crate::storage::{ContextQuery, StorageQueryMethod};
use crate::storage::StorageQuery;

#[derive(Clone)]
pub struct PageHandler {
    ptype: PageType,
}

impl Handler<RequestArgs> for PageHandler {
    type Output = HttpResponse<BoxBody>;
    type Future = Pin<Box<dyn Future<Output = Self::Output>>>;

    // Function called every time we have a request to handle
    fn call(&self, args: RequestArgs) -> Self::Future {
        log::debug!("Handling request with lang {:?}", args.lang);
        let default_template = self.ptype.default_template.clone();
        let add_ctxt = self.ptype.add_context.clone();
        let add_headers = self.ptype.add_headers.clone();

        let storage_query = match self.ptype.content_query.build_query(&self.ptype.storage, &args) {
            Ok(qry) => qry,
            Err(e) => return Box::pin(Self::error(args.render, e)),
        };

        Box::pin(Self::respond(
            storage_query,
            add_ctxt,
            add_headers,
            default_template,
            args,
        ))
    }
}

impl PageHandler {
    // Function called on initialization for each worker
    pub fn create(ptype: &PageType) -> PageHandler {
        PageHandler {
            ptype: ptype.clone(),
        }
    }

    pub async fn respond(
        mut qry: StorageQuery, // Content query
        add_ctxt: HashMap<String, ContextQuery>,
        add_headers: HashMap<String, String>,
        default_template: String,
        args: RequestArgs,
    ) -> HttpResponse<BoxBody> {
        // Fine tune content query
        if let Some(ref lang) = args.lang {
            qry.set_lang(lang.clone());
        }

        Self::build_response(
            args.render.clone(),
            add_headers,
            Self::handle_request(qry,
                &args,
                add_ctxt,
                default_template,
            ).await,
        )
        .await
    }

    pub async fn handle_request(
        qry: StorageQuery,
        args: &RequestArgs,
        add_ctxt: HashMap<String, ContextQuery>,
        default_template: String,
    ) -> Result<String, Errcode> {
        let mut ctxt = args.base_context.as_ref().clone();
        let (lang_opt, metadata, body) = if let StorageQueryMethod::NoOp = qry.method {
            (None, PageMetadata::default(), "".to_string())
        } else {
            args.storage.query(qry.clone()).await.page_content()?
        };

        ctxt.insert("id", &metadata.id);
        ctxt.insert("metadata", &metadata.metadata);
        ctxt.insert("route", &args.uri);
        if let Some(ref lang) = lang_opt {
            ctxt.insert("lang", lang);
        }

        insert_add_context(&add_ctxt, &metadata, &args, &mut ctxt).await?;
        insert_add_context(&metadata.add_context, &metadata, &args, &mut ctxt).await?;

        let template = if let Some(ref template) = metadata.template {
            template
        } else {
            &default_template
        };

        let res = args.render
            .render_content(template, body, &metadata, ctxt)
            .await?;
        Ok(res)
    }

    pub async fn error(render: Data<Render>, e: Errcode) -> HttpResponse<BoxBody> {
        let body = render.render_error(&e).await;
        let mut builder: HttpResponseBuilder = e.into();
        builder.body(body)
    }

    pub async fn build_response(
        render: Data<Render>,
        add_headers: HashMap<String, String>,
        body: Result<String, Errcode>,
    ) -> HttpResponse {
        match body {
            Ok(text) => {
                let mut reply = HttpResponse::Ok();
                for (key, val) in add_headers {
                    reply.append_header((key, val));
                }
                reply.body(text)
            }
            Err(e) => Self::error(render, e).await,
        }
    }
}

pub async fn insert_add_context(
    add_ctxt: &HashMap<String, ContextQuery>,
    page_md: &PageMetadata,
    args: &RequestArgs,
    ctxt: &mut Context
) -> Result<(), Errcode> {
    for (name, context_query) in add_ctxt {
        if let ContextQuery::Plain(d) = context_query {
            ctxt.insert(name, d);
            continue;
        }

        if let Some(mut qry) = context_query.get_storage_query(args, page_md)? {
            if let Some(ref lang) = args.lang {
                qry.set_lang(lang.clone());
            }

            let val = context_query.insert_data(name, ctxt, args.storage.query(qry).await)?;
        }
    }
    Ok(())
}
