use actix_files::Files;
use actix_web::http::header;
use actix_web::middleware::{Compress, DefaultHeaders, Logger};
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use clap::{arg, command, Parser};
use std::path::PathBuf;
use std::sync::Arc;

mod cache;
mod config;
mod endpoints;
mod errors;
mod loader;
mod post;
mod protection;
mod render;

use config::Configuration;

#[cfg(feature = "no_cache")]
const MAX_AGE: usize = 0;

#[cfg(not(feature = "no_cache"))]
const MAX_AGE: usize = 60 * 60;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Server configuration file
    #[arg(short, long, default_value = "config.toml")]
    config_file: PathBuf,

    /// Site configuration file
    #[arg(short, long, default_value = "site.toml")]
    site_config_file: PathBuf,

    /// Path to the favicon
    #[arg(short, long, default_value = "favicon.png")]
    favicon: PathBuf,

    /// Path to the directory containing SCSS code
    #[arg(long = "scss", default_value = "scss")]
    scss_dir: PathBuf,

    /// Path to the directory containing the Javascript code
    #[arg(long = "js", default_value = "js")]
    scripts_dir: PathBuf,

    /// Path to the directory containing the templates code
    #[arg(long = "html", default_value = "html")]
    templates_dir: PathBuf,

    /// Path to the directory where to store the generated assets
    #[arg(long = "out", default_value = "out")]
    assets_dir: PathBuf,

    /// Any additionnal path to add to the assets directory
    #[arg(long = "add")]
    add_assets: Vec<PathBuf>,

    /// Path to the posts directory
    #[cfg(feature = "local_storage")]
    #[arg(long = "posts", default_value = "posts")]
    posts_dir: PathBuf,

    /// Path to the file containing series definition
    #[cfg(feature = "local_storage")]
    #[arg(long = "series", default_value = "series.toml")]
    series_list: PathBuf,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Configuration::from(Args::parse());
    config.validate().expect("Invalid configuration");
    std::fs::create_dir_all(&config.assets_dir).expect("Unable to create assets dir");
    config.init_logging();
    let config = Arc::new(config);
    let port = config.server_port;

    let loader = Data::new(
        loader::Loader::init(config.clone()).expect("Error while initialization of Loader"),
    );
    let render = Data::new(
        render::Render::init(config.clone()).expect("Error while initialization of Render"),
    );
    std::fs::copy(&config.favicon, config.assets_dir.join("favicon"))?;
    fs_extra::copy_items(
        &config.add_assets,
        &config.assets_dir,
        &fs_extra::dir::CopyOptions::new().overwrite(true),
    )
    .expect("Unable to copy additionnal assets");
    let srv = HttpServer::new(move || {
        App::new()
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
            )
            // .wrap(protection::ProtectionMiddlewareBuilder::new(&config))
            .app_data(render.clone())
            .app_data(loader.clone())
            .configure(|srv| endpoints::configure(srv).expect("Unable to configure endpoints"))
            .service(Files::new("/", config.assets_dir.canonicalize().unwrap()))
    })
    .bind(("0.0.0.0", port))?
    .run();

    log::info!("Started on http://127.0.0.1:{port}");
    srv.await
}
