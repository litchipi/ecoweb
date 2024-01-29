use actix_web::web::ServiceConfig;

use crate::config::Configuration;
use crate::render::Render;

// TODO robots.txt

#[cfg(feature = "githook")]
pub mod githook;

#[cfg(feature = "humans-txt")]
pub mod humans;

#[cfg(feature = "rss")]
pub mod rss;

#[cfg(feature = "hireme")]
pub mod hireme;

#[cfg(feature = "webring")]
pub mod webring;

#[cfg(feature = "add-endpoint")]
pub mod addendpoint;

pub fn announce() {
    #[cfg(feature = "githook")]
    log::info!("Using extension githook");

    #[cfg(feature = "rss")]
    log::info!("Using extension RSS");

    #[cfg(feature = "humans-txt")]
    log::info!("Using extension humans.txt");

    #[cfg(feature = "hireme")]
    log::info!("Using extension hireme");

    #[cfg(feature = "webring")]
    log::info!("Using extension webring");

    #[cfg(feature = "add-endpoint")]
    log::info!("Using extension add-endpoint");
}

#[allow(unused_variables)]
pub fn configure(cfg: &Configuration, rdr: &Render, srv: &mut ServiceConfig) {
    #[cfg(feature = "githook")]
    {
        let secret = githook::GithookSecret::init();
        srv.app_data(actix_web::web::Data::new(secret))
            .service(githook::git_webhook);
    }

    #[cfg(feature = "rss")]
    srv.service(rss::get_rss_feed);

    #[cfg(feature = "humans-txt")]
    srv.service(humans::get_humans);

    #[cfg(feature = "hireme")]
    srv.service(hireme::get_hireme);

    #[cfg(feature = "add-endpoint")]
    srv.configure(|srv| {
        addendpoint::configure_add_endpoints(cfg, rdr, srv)
            .expect("Unable to configure additionnal endpoints")
    });
}
