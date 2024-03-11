use serde::{Deserialize, Serialize};

#[derive(Default, Hash, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct StorageQuery {
    pub lang_pref: Option<Vec<String>>,
    // TODO    Implement base storage queries here and in backend
    //    to decide what data has to be put inside the struct
}

impl StorageQuery {
    pub fn recent_pages(slug: &String, nb: usize) -> StorageQuery {
        StorageQuery::default()
    }

    pub fn content(slug: &String, id: Option<u64>) -> StorageQuery {
        StorageQuery::default()
    }

    pub fn set_lang(&mut self, lang: Vec<String>) {
        self.lang_pref = Some(lang);
    }
}
