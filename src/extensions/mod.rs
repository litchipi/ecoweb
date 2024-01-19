use actix_web::web::ServiceConfig;

use crate::errors::Errcode;

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
}

#[allow(unused_variables)]
pub fn configure(srv: &mut ServiceConfig) -> Result<(), Errcode> {
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

    Ok(())
}
