use std::path::PathBuf;

use actix_web::web::{Data, Bytes};
use actix_web::{post, HttpRequest, HttpResponse};
use sha2::Sha256;
use hmac::{Hmac, Mac};

use crate::config::Configuration;
use crate::errors::raise_error;
use crate::loader::Loader;
use crate::render::Render;
use crate::setup::reload;

pub struct WebhookSecret(Vec<u8>);
impl WebhookSecret {
    pub fn load(f: &PathBuf) -> WebhookSecret {
        let token = std::fs::read(f).expect("Unable to read webhook secret file");
        println!("{token:?}");
        WebhookSecret(token)
    }

    fn hmac(&self, data: &Bytes) -> String {
        let mut data_hmac : Hmac<Sha256> = hmac::Hmac::new_from_slice(&self.0).unwrap();
        data_hmac.update(data);
        hex::encode(data_hmac.finalize().into_bytes())
    }
}

#[post("/gitwebhook")]
async fn git_webhook(
    req: HttpRequest,
    data: Bytes,
    secret: Data<WebhookSecret>,
    ldr: Data<Loader>,
    rdr: Data<Render>,
    cfg: Data<Configuration>,
) -> HttpResponse {
    let t = std::time::Instant::now();
    let hmac = secret.hmac(&data);
    let header_hmac = req.headers().get("x-forgejo-signature").expect("Signature header not found").to_str().unwrap();
    println!("Webhook hmac: {hmac}, header hmac: {header_hmac}");
    if header_hmac != hmac {
        return HttpResponse::Forbidden().body("Wrong webhook secret")
    }
    // let json_data : serde_json::Value = serde_json::from_slice(&data.as_ref()).expect("Error while decoding json");
    if let Err(e) = reload(&ldr, &rdr, &cfg) {
        raise_error(e, &rdr)
    } else {
        HttpResponse::Ok().body(format!("Reloaded in {:?}", t.elapsed()))
    }
}

// "x-gogs-event": "push"
// "accept-encoding": "gzip"
// "host": "0.0.0.0:4446"
// "x-gitea-delivery": "57768f34-2ede-4813-a56f-8833c76acce1"
// "x-github-delivery": "57768f34-2ede-4813-a56f-8833c76acce1"
// "content-length": "5549"
// "user-agent": "Go-http-client/1.1"
// "x-forgejo-delivery": "57768f34-2ede-4813-a56f-8833c76acce1"
// "x-gitea-signature": "9490a08f7d3fd62ccacad6919c9600a0fd32c8ef36b610979934cf9df7e4af0c"
// "x-gitea-event-type": "push"
// "x-github-event": "push"
// "x-hub-signature": "sha1=0ad9b4820169217e89c97c7fac06b309d7e56733"
// "x-forgejo-event-type": "push"
// "x-gogs-signature": "9490a08f7d3fd62ccacad6919c9600a0fd32c8ef36b610979934cf9df7e4af0c"
// "x-forgejo-event": "push"
// "x-gogs-delivery": "57768f34-2ede-4813-a56f-8833c76acce1"
// "x-hub-signature-256": "sha256=9490a08f7d3fd62ccacad6919c9600a0fd32c8ef36b610979934cf9df7e4af0c"
// "x-gogs-event-type": "push"
// "x-gitea-event": "push"
// "x-forgejo-signature": "9490a08f7d3fd62ccacad6919c9600a0fd32c8ef36b610979934cf9df7e4af0c"
// "content-type": "application/json"
// "x-github-event-type": "push"
