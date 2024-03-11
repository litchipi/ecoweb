use actix_web::body::BoxBody;
use actix_web::web::Data;
use actix_web::{Handler, HttpResponse};
use tera::Context;
use std::pin::Pin;
use std::future::Future;
use std::collections::HashMap;

use super::data_extract::RequestArgs;
use crate::errors::Errcode;
use crate::page::PageType;
use crate::render::Render;
use crate::storage::{Storage, StorageQuery};
use crate::storage::ContextQuery;
use crate::routes::ContentQueryMethod;

#[derive(Clone)]
pub struct PageHandler {
    ptype: PageType,
}

impl Handler<RequestArgs> for PageHandler {
    type Output = HttpResponse<BoxBody>;
    type Future = Pin<Box<dyn Future<Output = Self::Output>>>;

    // Function called every time we have a request to handle
    fn call(&self, args: RequestArgs) -> Self::Future {
        let storage_query = match self.ptype.content_query {
            ContentQueryMethod::ContentId(ref slug) => {
                match args.get_query_id(slug) {
                    Ok(id) => StorageQuery::content(&self.ptype.storage, Some(id)),
                    Err(e) => {
                        return Box::pin(Self::error(args.render, e));
                    },
                }
            },
            ContentQueryMethod::FromSlug => {
                StorageQuery::content(&self.ptype.storage, None)
            }
        };
        Box::pin(Self::respond(storage_query, self.ptype.add_context.clone(), args))
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
        args: RequestArgs,
    ) -> HttpResponse<BoxBody> {
        if !args.storage.has_changed(&qry).await {
            if let Some(cached) = args.render.get_cache(&qry) {
                return Self::build_response(args.render, Ok(cached)).await;
            }
        }

        // Build context
        let mut ctxt = args.base_context.as_ref().clone();
        for (name, data) in add_ctxt {
            if let Err(e) = data.insert_context(&args.storage, &name, &mut ctxt).await {
                return Self::error(args.render, e).await;
            }
        }

        // Fine tune content query
        if let Some(lang) = args.lang {
            qry.set_lang(lang);
        }

        // TODO    Check if template is already loaded in engine or not
        // TODO    Load template from storage if not loaded, and add to engine

        Self::build_response(args.render.clone(),
            Self::handle_request(qry, &args.render, &args.storage, ctxt).await
        ).await
    }

    pub async fn handle_request(qry: StorageQuery, render: &Render, storage: &Storage, mut ctxt: Context) -> Result<String, Errcode> {
        let (metadata, body) = storage
            .query(qry).await
            .page_content()?;

        for (name, data) in metadata.add_context.iter() {
            data.insert_context(storage, name, &mut ctxt).await?;
        }
        ctxt.insert("page-content", &body);

        // TODO    Render post based on data
        Ok(format!("{ctxt:?}"))
    }

    pub async fn error(render: Data<Render>, e: Errcode) -> HttpResponse<BoxBody> {
        let mut builder = match e {
            Errcode::ParameterNotInUrl => HttpResponse::NotFound(),
            Errcode::DataNotFound(_) => HttpResponse::NotFound(),
            Errcode::NoRecentPagesFound(_) => HttpResponse::NotFound(),
            _ => HttpResponse::InternalServerError(),
        };
        builder.body(render.render_error(e).await)
    }

    pub async fn build_response(render: Data<Render>, body: Result<String, Errcode>) -> HttpResponse {
        // TODO    Additionnal headers here
        match body {
            Ok(text) => HttpResponse::Ok().body(text),
            Err(e) => Self::error(render, e).await,
        }
    }
}
