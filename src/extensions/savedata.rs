use std::path::PathBuf;
use std::time::Duration;

use actix_web::web::{Data, Form, Path};
use actix_web::{post, HttpResponse};
use serde_json::Value;

use crate::{
    config::Configuration,
    errors::{raise_error, Errcode},
    render::Render,
};

// TODO Check on startup that the directory exists (or create if not),
// and has all the permissions required to write inside

const UNAUTHORIZED_PENALTY_MS: u64 = 1500;
const NOTIF_MSG: &str = "Your data is now saved, you can go back to the website";

fn check_allowed(data: &Value, config: &Configuration) -> bool {
    let Some(s) = data.as_object().unwrap().get("save-data-token") else {
        return false;
    };

    let Some(token) = s.as_str() else {
        return false;
    };

    config
        .site_config
        .allowed_savedata_tokens
        .contains(&token.to_string())
}

#[post("/savedata/{slug}")]
async fn post_savedata(
    path: Path<(String,)>,
    data: Form<Value>,
    cfg: Data<Configuration>,
    rdr: Data<Render>,
) -> HttpResponse {
    assert!(data.is_object(), "Wrong format of data");
    let slug = &path.0;
    let outdir = cfg.save_data_dir.join(slug);
    if !check_allowed(&data, &cfg) {
        let page = rdr.render_error("Not authorized to access this page");
        tokio::time::sleep(Duration::from_millis(UNAUTHORIZED_PENALTY_MS)).await;
        return HttpResponse::Unauthorized().body(page);
    }
    match save_data(outdir, &data).await {
        Ok(()) => {
            let mut ctxt = rdr.base_context.clone();
            ctxt.insert("notif_h1", "Thank you");
            match rdr.render_notification("Data saved", NOTIF_MSG, ctxt) {
                Ok(page) => HttpResponse::Ok().body(page),
                Err(e) => raise_error(e, &rdr),
            }
        }
        Err(e) => raise_error(e, &rdr),
    }
}

async fn save_data(out: PathBuf, data: &Value) -> Result<(), Errcode> {
    log::debug!("Savedata submited: {data:?}");
    tokio::fs::create_dir_all(&out).await?;
    let jsonstring = serde_json::to_string(data)?;
    let nexist = std::fs::read_dir(&out)?.count();
    tokio::fs::write(
        out.join(nexist.to_string()).with_extension("json"),
        jsonstring,
    )
    .await?;
    Ok(())
}
