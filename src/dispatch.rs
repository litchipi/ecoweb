use actix_web::dev::fn_factory;
use actix_web::web::{self, Data, ServiceConfig};
use actix_web::{FromRequest, HttpRequest, Responder};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::page::PageType;
use crate::render::{Render, render_markdown};
use crate::storage::Storage;

macro_rules! create_endpoint {
    ($config:expr, $ptype:expr) => {
        move |(req, ldr, rdr): (HttpRequest, Data<Storage>, Data<Render>)| {
            let ptype = $ptype.clone();
            let default_lang = $config.default_lang.clone();
            async {
                let content_query = if ptype.lang_detect {
                    if let Some(lang_prefs) = get_lang(&req) {
                        lang_prefs.push(default_lang);
                        ptype.build_query_with_lang(&req, lang_prefs)
                    } else {
                        ptype.build_query(&req)
                    }
                } else {
                    ptype.build_query(&req)
                };

                if let Some(page) = rdr.get_cache(&content_query) {
                    page.clone()
                } else {
                    let (metadata, markdown) = if !ldr.has_changed(&content_query) {
                        ldr.query_cache(&content_query)
                    } else {
                        ldr.query(&content_query).page_content().unwrap()
                    };
                    rdr.add_template(&ldr, &ptype, &metadata);
                    let mut ctxt = rdr.build_context(&ldr, &metadata, &ptype);
                    let html_content = render_markdown(markdown, &mut ctxt);
                    rdr.render_content(html_content, &metadata, &ptype, &ctxt)
                }
            }
        }
    };
}
