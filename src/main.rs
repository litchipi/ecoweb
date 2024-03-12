#![allow(dead_code, unused_variables)] // TODO    Remove once dev finished
use actix_web::middleware::{Compress, Logger};
use actix_web::{web::Data, App, HttpServer};

mod cache;
mod config;
mod errors;
mod page;
mod render;
mod routes;
mod storage;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = config::Config::load().expect("Unable to load server configuration");
    config.setup_logging();

    let storage = Data::new(storage::Storage::init(&config).expect("Unable to initialize storage"));
    let render = Data::new(
        render::Render::init(storage.clone().into_inner(), &config)
            .await
            .expect("Error while initializing render engine"),
    );
    let base_context = Data::new(config.base_templating_context());

    let port = config.server_port;

    let srv = HttpServer::new(move || {
        let app = App::new()
            .wrap(Compress::default())
            .wrap(Logger::new("%s | %r (%bb in %Ts) from %a"))
            .wrap(config.get_default_headers())
            .app_data(base_context.clone())
            .app_data(storage.clone())
            .app_data(render.clone());

        app.configure(|app| {
            routes::configure(&config, app);
        })
    });

    let srv = srv.bind(("0.0.0.0", port))?.run();
    log::info!("Serving content on http://0.0.0.0:{port}");
    srv.await
}
