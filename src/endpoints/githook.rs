use std::io::Read;

use actix_web::web::{Bytes, Data};
use actix_web::{post, HttpRequest, HttpResponse};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use crate::config::Configuration;
use crate::errors::Errcode;
use crate::loader::Loader;
use crate::render::Render;
use crate::setup::reload;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    zipfile_fetch_url: String,
}

pub struct WebhookSecret(Hmac<Sha256>);
impl WebhookSecret {
    pub fn init() -> WebhookSecret {
        let secret = std::env::var("GIT_WEBHOOK_SECRET")
            .expect("Unable to get webhook secret from GIT_WEBHOOK_SECRET env var");
        WebhookSecret(
            hmac::Hmac::new_from_slice(secret.as_bytes()).expect("Unable to create HMAC generator"),
        )
    }

    fn hmac(&self, data: &Bytes) -> String {
        let mut data_hmac = self.0.clone();
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
    let Some(header_hmac) = req
        .headers()
        .get("x-forgejo-signature")
        .map(|h| h.to_str().ok())
        .flatten()
    else {
        return HttpResponse::BadRequest().body("Signature header not found or invalid");
    };
    if header_hmac != hmac {
        return HttpResponse::Forbidden().body("Wrong webhook secret");
    }

    let Ok(json_data) = serde_json::from_slice::<serde_json::Value>(&data.as_ref()) else {
        return HttpResponse::BadRequest().body("Unable to decode JSON from payload data");
    };

    let Some(head_commit) = json_data
        .get("head_commit")
        .map(|m| m.as_object())
        .flatten()
    else {
        return HttpResponse::BadRequest().body("No key \"head_commit\" in JSON data, or invalid");
    };

    let Some(commit_id) = head_commit.get("id").map(|i| i.as_str()).flatten() else {
        return HttpResponse::BadRequest()
            .body("No key \"id\" in map \"head_commit\" in JSON data, or invalid");
    };

    if let Err(e) = download_latest_data(commit_id, &cfg).await {
        return HttpResponse::InternalServerError()
            .body(format!("Error while fetching latest data: {e:?}"));
    }

    match reload(&ldr, &rdr, &cfg) {
        Err(e) => HttpResponse::InternalServerError().body(format!("{e:?}")),
        Ok(()) => HttpResponse::Ok().body(format!("Done in {:?}", t.elapsed())),
    }
}

fn get_basedir(acc: &String, x: &str) -> String {
    let mut res = "".to_string();
    for (c1, c2) in acc.chars().zip(x.chars()) {
        if c1 == c2 {
            res.push(c1);
        } else {
            return res;
        }
    }
    res
}

async fn download_latest_data(id: &str, cfg: &Configuration) -> Result<(), Errcode> {
    let whcfg = &cfg.site_config.webhook_update;
    log::debug!("Download data from commit id {id}");
    let url = format!("{}/{id}.zip", whcfg.zipfile_fetch_url);
    log::debug!("{url}");
    let client = awc::Client::default();

    let mut data = client.get(url).send().await?;
    if !data.status().is_success() {
        return Err(Errcode::ErrorStatusCode(data.status()));
    }
    let data = std::io::Cursor::new(data.body().await?);
    std::fs::write("/tmp/archive.zip", data.get_ref()).unwrap();
    let mut zip_archive = zip::ZipArchive::new(data)?;

    let all_files = zip_archive
        .file_names()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    let base_dir = zip_archive
        .file_names()
        .fold(None, |acc, x| {
            if let Some(acc) = acc {
                Some(get_basedir(&acc, x))
            } else {
                Some(x.to_string())
            }
        })
        .unwrap();

    for file in all_files {
        let filetail = file.strip_prefix(&base_dir).unwrap();
        if filetail.is_empty() {
            continue;
        }
        let fpath = cfg.data_dir.join(filetail);
        let mut f = zip_archive.by_name(&file).unwrap();
        if f.is_file() {
            log::debug!("Update {fpath:?}...");
            std::fs::create_dir_all(fpath.parent().unwrap()).unwrap();
            let mut buffer = Vec::with_capacity(f.size() as usize);
            f.read_to_end(&mut buffer).unwrap();
            std::fs::write(fpath, buffer).unwrap();
        }
    }

    Ok(())
}
