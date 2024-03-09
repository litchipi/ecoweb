use actix_web::middleware::{Compress, Logger};
use actix_web::{web::Data, App, HttpServer};

mod errors;
mod page;
mod storage;
mod render;
mod config;
mod routes;
mod cache;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = config::Config::init();
    config.setup_logging();

    let storage = Data::new(storage::Storage::init(&config));
    let render = Data::new(render::Render::init(&config));

    let port = config.server_port;

    let srv = HttpServer::new(move || {
        let app = App::new()
            .wrap(Compress::default())
            .wrap(Logger::new("%s | %r (%bb in %Ts) from %a"))
            .wrap(config.get_default_headers())
            // TODO    Add base context for template rendering
            .app_data(storage.clone())
            .app_data(render.clone());
        app.configure(|app| routes::configure(&config, app))
    });
    let srv = srv
        .bind(("0.0.0.0", port))?
        .run();
    
    // log::info!("Serving content on http://0.0.0.0:{port}");
    println!("Serving content on http://0.0.0.0:{port}");
    srv.await
}
