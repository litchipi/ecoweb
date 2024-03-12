use std::sync::Arc;

use actix_web::{HttpResponse, HttpResponseBuilder};

use crate::storage::StorageErrorType;

#[derive(Clone, Debug)]
pub enum Errcode {
    // General
    FilesystemError(&'static str, Arc<std::io::Error>),

    // Configuration
    ConfigFileRead(Arc<std::io::Error>),

    // Data extraction
    ContentIdParsing(std::num::ParseIntError),
    ParameterNotInUrl,

    // Storage
    StorageError(StorageErrorType),
    WrongStorageData(&'static str),

    // Render
    RegisterTemplate(String),

    // Serialization
    TomlDecode(&'static str, toml::de::Error),
}

impl Into<HttpResponseBuilder> for Errcode {
    fn into(self) -> HttpResponseBuilder {
        match self {
            Errcode::ParameterNotInUrl => HttpResponse::NotFound(),
            Errcode::StorageError(e) => e.into(),
            _ => HttpResponse::InternalServerError(),
        }
    }
}

impl From<tera::Error> for Errcode {
    fn from(value: tera::Error) -> Self {
        Errcode::RegisterTemplate(format!("{value:?}"))
    }
}
