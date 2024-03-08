use serde::{Deserialize, Serialize};

#[derive(Default, Hash, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct StorageQuery {
    pub lang_pref: Option<Vec<String>>,
}

impl StorageQuery {
    pub fn recent_pages(ptype: &String, nb: usize) -> StorageQuery {
        // TODO    Implement this storage query here and in backend
        //    to decide what data has to be put inside the struct
        StorageQuery::default()
    }
}
