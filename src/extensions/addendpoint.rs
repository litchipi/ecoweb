use actix_web::web::{Data, ServiceConfig};
use actix_web::{web, HttpResponse};
use tera::Context;

use crate::config::Configuration;
use crate::errors::{raise_error, Errcode};
use crate::render::Render;

fn slug_from_path(path: String) -> String {
    path.replace("/", "_")
}

pub async fn insert_additionnal_context(
    config: &Configuration,
    ctxt: &mut Context,
) -> Result<(), Errcode> {
    for (name, fpath) in config.site_config.additionnal_context.iter() {
        let fpath = config.data_dir.join(fpath);
        let data: toml::Value = toml::from_str(tokio::fs::read_to_string(fpath).await?.as_str())?;
        ctxt.insert(name, &data);
    }
    Ok(())
}

async fn add_endpoint(path: String, rdr: Data<Render>) -> HttpResponse {
    let mut ctxt = rdr.base_context.clone();
    ctxt.insert("add_endpoint_path", &path);
    match rdr.render(&slug_from_path(path), &ctxt) {
        Err(e) => raise_error(e, &rdr),
        Ok(res) => HttpResponse::Ok().body(res),
    }
}

pub fn register_templates(config: &Configuration, rdr: &Render) -> Result<(), Errcode> {
    for (path, template_file) in config.add_endpoints.clone().into_iter() {
        rdr.engine.write().add_template_file(
            config.templates_dir.join(template_file),
            Some(slug_from_path(path.clone()).as_str()),
        )?;
    }
    Ok(())
}

pub fn configure_add_endpoints(
    config: &Configuration,
    rdr: &Render,
    srv: &mut ServiceConfig,
) -> Result<(), Errcode> {
    register_templates(config, rdr)?;
    for (path, _) in config.add_endpoints.clone().into_iter() {
        let path2 = path.clone();
        let route = web::get().to(move |rdr: Data<Render>| add_endpoint(path2.clone(), rdr));
        srv.route(path.as_str(), route);
    }
    Ok(())
}
