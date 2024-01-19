use actix_files::Files;
use actix_web::http::header;
use actix_web::middleware::{Compress, DefaultHeaders, Logger};
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use clap::{arg, command, Parser};
use std::path::PathBuf;
use std::sync::Arc;

#[cfg(feature="hot_reloading")]
use actix_web::dev::Service;

mod config;
mod endpoints;
mod errors;
mod loader;
mod post;
mod render;
mod setup;

use config::Configuration;

use crate::endpoints::githook::WebhookSecret;

#[cfg(feature = "no_cache")]
const MAX_AGE: usize = 0;

#[cfg(not(feature = "no_cache"))]
const MAX_AGE: usize = 60 * 60;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path where the blog data is located
    #[arg(short, long)]
    data_dir: PathBuf,

    /// Path to the directory where to store the generated assets
    #[arg(long = "out", default_value = "out")]
    assets_dir: PathBuf,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let config = Configuration::from(args);
    config.validate().expect("Invalid configuration");
    config.init_logging();
    setup::setup_files(&config).expect("Unable to setup files");
    let webhook_secret = Data::new(WebhookSecret::init());


    let config = Arc::new(config);
    let port = config.server_port;

    let loader = Data::new(
        loader::Loader::init(config.clone()).expect("Error while initialization of Loader"),
    );
    let loader_cpy = loader.clone();
    let render = Data::new(
        render::Render::init(config.clone()).expect("Error while initialization of Render"),
    );
    let srv = HttpServer::new(move || {
        let app = App::new()
            .wrap(Compress::default())
            .wrap(Logger::new("%s | %r (%bb in %Ts) from %a"))
            .wrap(
                DefaultHeaders::new()
                    .add((header::X_FRAME_OPTIONS, "DENY"))
                    .add((header::CONTENT_TYPE, "text/html; charset=UTF-8"))
                    .add((header::X_CONTENT_TYPE_OPTIONS, "nosniff"))
                    .add((
                        header::PERMISSIONS_POLICY,
                        "geolocation=(), camera=(), microphone=()",
                    ))
                    .add((header::CACHE_CONTROL, format!("max-age={MAX_AGE}")))
                    .add((header::AGE, "0")),
            );

        #[cfg(feature = "hot_reloading")]
        let app = {
            let loader = loader.clone();
            let render = render.clone();
            let config = config.clone();
            app.wrap_fn(move |req, srv| {
                let path = req.path();
                if path.starts_with("/post/") || (path == "/") || (path == "/allposts") {
                    setup::reload(&loader, &render, &config).expect("Unable to reload data");
                }
                srv.call(req)
            })
        };
        // .wrap(protection::ProtectionMiddlewareBuilder::new(&config))

        app.app_data(render.clone())
            .app_data(loader.clone())
            .app_data(Data::from(config.clone()))
            .app_data(webhook_secret.clone())
            .configure(|srv| endpoints::configure(srv).expect("Unable to configure endpoints"))
            .service(Files::new("/", config.assets_dir.canonicalize().unwrap()))
    })
    .bind(("0.0.0.0", port))?
    .run();

    log::info!("Serving content on http://0.0.0.0:{port}");
    let res = srv.await;

    log::info!("Stopping additionnal workers");
    match Arc::into_inner(loader_cpy.into_inner()).map(|l| l.clean_exit()) {
        Some(Ok(())) => log::debug!("Loader exitted clean"),
        Some(Err(e)) => log::error!("Error during exit clean of the Loader: {e:?}"),
        None => log::error!("Unable to get a single reference of loader after exit"),
    }

    res
}
