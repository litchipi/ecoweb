use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct WebringConfig {
    name: String,
    next: String,
    previous: String,
}
