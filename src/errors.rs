use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum Errcode {
    // General
    FilesystemError(&'static str, Arc<std::io::Error>),

    // Configuration
    ConfigFileRead(Arc<std::io::Error>),

    // ContextQuery
    NoRecentPagesFound(String),

    // Data extraction
    ContentIdParsing(std::num::ParseIntError),
    ParameterNotInUrl,

    // Storage
    WrongStorageData(&'static str),
    LangNotSupported(Vec<String>),
    DataNotFound(String),
    ContentMalformed(&'static str),

    // Serialization
    TomlDecode(&'static str, toml::de::Error),
}
