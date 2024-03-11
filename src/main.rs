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

    let test = storage::ContextQuery::RecentPages("toto".to_string(), 3);
    let test2 = storage::ContextQuery::Plain(serde_json::json!("toto"));
    println!("{}", toml::to_string(&test).unwrap());
    println!("{}", toml::to_string(&test2).unwrap());
    let mut hmap = std::collections::HashMap::new();
    hmap.insert("test", test);
    hmap.insert("test2", test2);
    println!("{}", toml::to_string(&hmap).unwrap());
    
    let config = config::Config::load().expect("Unable to load server configuration");
    config.setup_logging();
    log::debug!("Configuration:\n{config:?}");

    let storage = Data::new(storage::Storage::init(&config));
    let render = Data::new(render::Render::init(&config));
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

    // log::info!("Serving content on http://0.0.0.0:{port}");
    log::info!("Serving content on http://0.0.0.0:{port}");
    srv.await
}
