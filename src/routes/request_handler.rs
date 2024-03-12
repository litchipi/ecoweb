use actix_web::body::BoxBody;
use actix_web::web::Data;
use actix_web::{Handler, HttpResponse, HttpResponseBuilder};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use tera::Context;

use super::data_extract::RequestArgs;
use crate::errors::Errcode;
use crate::page::PageType;
use crate::render::Render;
use crate::storage::ContextQuery;
use crate::storage::{Storage, StorageQuery};

#[derive(Clone)]
pub struct PageHandler {
    ptype: PageType,
}

impl Handler<RequestArgs> for PageHandler {
    type Output = HttpResponse<BoxBody>;
    type Future = Pin<Box<dyn Future<Output = Self::Output>>>;

    // Function called every time we have a request to handle
    fn call(&self, args: RequestArgs) -> Self::Future {
        let storage_query = match self.ptype.content_query.build_query(&self.ptype.storage, &args) {
            Ok(qry) => qry,
            Err(e) => return Box::pin(Self::error(args.render, e)),
        };
        let default_template = self.ptype.default_template.clone();
        let add_ctxt = self.ptype.add_context.clone();
        Box::pin(Self::respond(
            storage_query,
            add_ctxt,
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
        default_template: String,
        args: RequestArgs,
    ) -> HttpResponse<BoxBody> {
        // TODO    This function has to be only when nothing changed, including context
        if !args.storage.has_changed(&qry).await {
            if let Some(cached) = args.render.cache.get(&qry) {
                return Self::build_response(args.render, Ok(cached)).await;
            }
        }

        // Fine tune content query
        if let Some(lang) = args.lang {
            qry.set_lang(lang);
        }

        Self::build_response(
            args.render.clone(),
            Self::handle_request(qry,
                &args.render,
                &args.storage,
                add_ctxt,
                default_template,
                args.base_context.as_ref().clone(),
            ).await,
        )
        .await
    }

    pub async fn handle_request(
        qry: StorageQuery,
        render: &Render,
        storage: &Storage,
        add_ctxt: HashMap<String, ContextQuery>,
        default_template: String,
        mut ctxt: Context,
    ) -> Result<String, Errcode> {
        let (metadata, body) = storage.query(qry.clone()).await.page_content()?;

        ctxt.insert("id", &metadata.id);
        ctxt.insert("metadata", &metadata.metadata);
        for (name, data) in add_ctxt {
            data.insert_context(storage, &name, &metadata, &mut ctxt).await?;
        }
        for (name, data) in metadata.add_context.iter() {
            data.insert_context(storage, name, &metadata, &mut ctxt).await?;
        }

        let template = if let Some(ref template) = metadata.template {
            template
        } else {
            &default_template
        };

        let res = render
            .render_content(template, body, &metadata, ctxt)
            .await?;
        render.cache.add(qry, res.clone());
        Ok(res)
    }

    pub async fn error(render: Data<Render>, e: Errcode) -> HttpResponse<BoxBody> {
        let body = render.render_error(&e).await;
        let mut builder: HttpResponseBuilder = e.into();
        builder.body(body)
    }

    pub async fn build_response(
        render: Data<Render>,
        body: Result<String, Errcode>,
    ) -> HttpResponse {
        // TODO    Additionnal headers here
        match body {
            Ok(text) => HttpResponse::Ok().body(text),
            Err(e) => Self::error(render, e).await,
        }
    }
}
