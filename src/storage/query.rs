use std::{collections::hash_map::DefaultHasher, hash::Hasher};

use serde::{Deserialize, Serialize};

#[repr(u8)]
#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize, Clone)]
/// All the methods that a storage have to implement in order to work
pub enum StorageQueryMethod {
    #[default]
    NoOp = 0,
    ContentNoId,
    ContentNumId(u64),
    GetRecentPages,
}

impl StorageQueryMethod {
    pub fn build_query(self, slug: &String) -> StorageQuery {
        let mut qry = StorageQuery {
            storage_slug: slug.clone(),
            method: self,
            ..Default::default()
        };
        qry.update_key();
        qry
    }
}

#[derive(Debug, Default, Eq, Clone)]
pub struct StorageQuery {
    key: u64,
    pub storage_slug: String,
    pub method: StorageQueryMethod,
    pub limit: usize,
    pub lang_pref: Option<Vec<String>>,
}

impl std::hash::Hash for StorageQuery {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.key)
    }
}

impl std::cmp::PartialEq for StorageQuery {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl StorageQuery {
    pub fn recent_pages(slug: &String, nb: usize) -> StorageQuery {
        let mut qry = StorageQueryMethod::GetRecentPages.build_query(slug);
        qry.limit = nb;
        qry.update_key();
        qry
    }

    pub fn content(slug: &String, id: Option<u64>) -> StorageQuery {
        let method = if let Some(id) = id {
            StorageQueryMethod::ContentNumId(id)
        } else {
            StorageQueryMethod::ContentNoId
        };
        method.build_query(slug)
    }

    pub fn update_key(&mut self) {
        let mut s = DefaultHasher::new();
        s.write(self.storage_slug.as_bytes());
        // s.write_u8(self.method as u8);    // TODO Make this work
        match self.method {
            StorageQueryMethod::NoOp => s.write_u8(0),
            StorageQueryMethod::ContentNoId => s.write_u8(1),
            StorageQueryMethod::ContentNumId(id) => {
                s.write_u8(2);
                s.write_u64(id);
            }
            StorageQueryMethod::GetRecentPages => s.write_u8(3),
        }
        s.write_usize(self.limit);
        self.key = s.finish();
    }

    pub fn set_lang(&mut self, lang: Vec<String>) {
        self.lang_pref = Some(lang);
    }
}
