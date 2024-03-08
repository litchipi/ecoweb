use actix_web::web::ServiceConfig;
use serde::{Deserialize, Serialize};

use crate::config::Config;

#[derive(Clone, Serialize, Deserialize)]
pub struct UploadEndpoint {
    pub route: String,
}

pub fn setup_routes(cfg: &Config, app: &mut ServiceConfig) {
    // TODO    Create upload routes
    // for endpoint in cfg.upload_endpoints.iter() {
    //     app.route(
    //         endpoint.route.as_str(),
    //         web::post().to(
    //             todo!() // Create an upload route
    //         )
    //     );
    // }
}
