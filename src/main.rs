#![allow(dead_code, unreachable_code)]

mod page;
mod storage;
mod render;
mod dispatch;
mod config;
mod upload;

fn main() {
    // Get path from args, toml from path, config from toml
    let mut config = config::Config::init();
    config.override_env_vars();
    let storage = storage::Storage::init(&config);
    let render = render::Render::init(&storage, &config);

    let mut app = config.setup_app_base();
    dispatch::create_endpoints(&config, &mut app);
    dispatch::setup_static_files_endpoint(&config, &mut app);
    // app.app_data(Data::new(storage));
    // app.app_data(Data::new(render));
}
