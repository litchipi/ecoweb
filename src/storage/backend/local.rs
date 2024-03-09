use std::collections::HashMap;

use crate::{config::Config, storage::{ContextQuery, PageMetadata, StorageData, StorageQuery}};

use super::StorageBackend;


pub struct LocalStorage {
    
}

impl StorageBackend for LocalStorage {
    fn init(config: &Config) -> Self where Self: Sized {
        // TODO    Get local storage config from config file
        LocalStorage {}
    }

    async fn has_changed(&self, qry: &StorageQuery) -> bool {
        false
    }

    // TODO    Match the query, and dispatch to commands
    async fn query(&self, qry: &StorageQuery) -> StorageData {
        let mut metadata = PageMetadata {
            add_context: HashMap::new(),
        };
        metadata.add_context.insert(
            "local_storage".to_string(),
            ContextQuery::Plain(serde_json::Value::Bool(true))
        );

        StorageData::PageContent {
            metadata,
            body: "body".to_string(),
        }
    }
}
