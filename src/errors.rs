use std::sync::Arc;

use actix_web::{HttpResponse, HttpResponseBuilder};

use crate::storage::StorageErrorType;

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
    ContextQueryBuild(&'static str, String),

    // Render
    RegisterTemplate(String),
    MarkdownRender(mdtrans::Errcode),

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

impl From<mdtrans::Errcode> for Errcode {
    fn from(value: mdtrans::Errcode) -> Self {
        Errcode::MarkdownRender(value)
    }
}

impl std::fmt::Debug for Errcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Errcode::TomlDecode(what, err) => {
                writeln!(f, "Error decoding {what}:")?;
                write!(f, "{}", err.to_string())
            }
            _ => write!(f, "{:?}", self),
        }
    }
}
