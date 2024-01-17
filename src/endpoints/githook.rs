use actix_web::web::{Data, Json};
use actix_web::{post, HttpRequest, HttpResponse};

use crate::config::Configuration;
use crate::errors::raise_error;
use crate::loader::Loader;
use crate::render::Render;
use crate::setup::reload;

type WebhookData = serde_json::Value;

#[post("/gitwebhook")]
async fn git_webhook(
    req: HttpRequest,
    data: Json<WebhookData>,
    ldr: Data<Loader>,
    rdr: Data<Render>,
    cfg: Data<Configuration>,
) -> HttpResponse {
    let t = std::time::Instant::now();
    log::debug!("Request:\n{req:?}");
    log::debug!(
        "Data from git webhook:\n{}",
        serde_json::to_string_pretty(&data).unwrap()
    );
    if let Err(e) = reload(&ldr, &rdr, &cfg) {
        raise_error(e, &rdr)
    } else {
        HttpResponse::Ok().body(format!("Reloaded in {:?}", t.elapsed()))
    }
}
