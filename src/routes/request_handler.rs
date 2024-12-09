use actix_web::body::BoxBody;
use actix_web::web::Data;
use actix_web::{Handler, HttpResponse};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use tera::Context;

use super::data_extract::RequestArgs;
use crate::errors::Errcode;
use crate::page::{PageMetadata, PageType};
use crate::render::Render;
use crate::storage::StorageQuery;
use crate::storage::{ContextQuery, StorageQueryMethod};

#[derive(Clone)]
pub struct PageHandler {
    ptype: PageType,
    default_lang: String,
}

impl Handler<RequestArgs> for PageHandler {
    type Output = HttpResponse<BoxBody>;
    type Future = Pin<Box<dyn Future<Output = Self::Output>>>;

    // Function called every time we have a request to handle
    fn call(&self, mut args: RequestArgs) -> Self::Future {
        log::debug!("Handling request with lang {:?}", args.lang);
        if args.lang.is_none() {
            args.lang = Some(vec![self.default_lang.clone()]);
        }
        let default_template = self.ptype.default_template.clone();
        let add_ctxt = self.ptype.add_context.clone();
        let mut add_headers = self.ptype.add_headers.clone();
        let cache_max_age = self.ptype.cache_max_age;
        add_headers.insert(
            "Cache-Control".to_string(),
            format!("max_age={cache_max_age}"),
        );

        let storage_query = match self
            .ptype
            .content_query
            .build_query(&self.ptype.storage, &args)
        {
            Ok(qry) => qry,
            Err(e) => return Box::pin(e.build_http_response_from_data(args.render, args.ctxt)),
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
    pub fn create(ptype: &PageType, default_lang: &str) -> PageHandler {
        PageHandler {
            ptype: ptype.clone(),
            default_lang: default_lang.to_owned(),
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
            Self::handle_request(qry, &args, add_ctxt, default_template).await,
            &args.ctxt,
        )
        .await
    }

    pub async fn handle_request(
        qry: StorageQuery,
        args: &RequestArgs,
        add_ctxt: HashMap<String, ContextQuery>,
        default_template: String,
    ) -> Result<String, Errcode> {
        let mut ctxt = args.ctxt.clone();
        let (lang_opt, metadata, body) = if let StorageQueryMethod::NoOp = qry.method {
            (None, PageMetadata::default(), "".to_string())
        } else {
            let (l, md, b) = args.storage.query(qry.clone()).await.page_content()?;
            ctxt.insert("id", &md.id);
            ctxt.insert("metadata", &md.metadata);
            (l, md, b)
        };

        ctxt.insert("route", &args.uri);

        // Lang that the data from storage is written in
        if let Some(ref lang) = lang_opt {
            ctxt.insert("lang", lang);
        }

        insert_add_context(&add_ctxt, &metadata, args, &mut ctxt).await?;
        insert_add_context(&metadata.add_context, &metadata, args, &mut ctxt).await?;

        let template = if let Some(ref template) = metadata.template {
            template
        } else {
            &default_template
        };

        let res = args.render.render_content(template, body, ctxt).await?;

        #[cfg(feature = "html_minify")]
        if metadata.minify {
            if let Ok(minpage) = args.render.minify(&res) {
                return Ok(minpage);
            } else {
                log::warn!("Unable to minify page {}", args.uri);
            }
        }

        Ok(res)
    }

    pub async fn build_response(
        render: Data<Render>,
        add_headers: HashMap<String, String>,
        body: Result<String, Errcode>,
        base_context: &Context,
    ) -> HttpResponse {
        match body {
            Ok(text) => {
                let mut reply = HttpResponse::Ok();
                for (key, val) in add_headers {
                    reply.append_header((key, val));
                }
                reply.body(text)
            }
            Err(e) => e.build_http_response(&render, base_context.clone()).await,
        }
    }
}

pub async fn insert_add_context(
    add_ctxt: &HashMap<String, ContextQuery>,
    page_md: &PageMetadata,
    args: &RequestArgs,
    ctxt: &mut Context,
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

            context_query.insert_data(name, ctxt, args.storage.query(qry).await)?;
        }
    }
    Ok(())
}
