use actix_web::http::header;
use actix_web::web::Data;
use actix_web::{get, HttpResponse};

use crate::config::{Configuration, SiteConfig};

#[get("/humans.txt")]
async fn get_humans(cfg: Data<Configuration>) -> HttpResponse {
    HttpResponse::Ok()
        .append_header((header::CONTENT_TYPE, "text/plain"))
        .body(cfg.site_config.humans_txt.clone())
}

pub fn generate_humans_txt(cfg: &mut SiteConfig) {
    cfg.humans_txt = String::new();
    cfg.humans_txt += "/* TEAM */\n";
    cfg.humans_txt += format!("Author: {}\n", cfg.author_name).as_str();
    for (sitename, social) in cfg.social.iter() {
        if sitename == "email" {
            let address = cfg.author_email.replace('@', " [at] ");
            cfg.humans_txt += format!("Email: {}\n", address).as_str();
        } else {
            let mut s = sitename.chars();
            let sitename_cap = match s.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + s.as_str(),
            };
            cfg.humans_txt += format!("{}: {}\n", sitename_cap, social).as_str();
        }
    }
    if let Some(ref blog_engine) = cfg.blog_engine_src {
        cfg.humans_txt += format!("\nSoftware sources: {}\n", blog_engine).as_str();
    }
    if let Some(ref blog_src) = cfg.blog_src {
        cfg.humans_txt += format!("Content sources: {}\n", blog_src).as_str();
    }
    cfg.humans_txt += "\nLanguage: English\n";
}
