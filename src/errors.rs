#[derive(Debug)]
pub enum Errcode {
    // Configuration
    ConfigFileRead(std::io::Error),

    // ContextQuery
    NoRecentPagesFound(String),

    // Data extraction
    ContentIdParsing(std::num::ParseIntError),
    ParameterNotInUrl,

    // StorageData
    WrongStorageData(&'static str),

    // Serialization
    TomlDecode(&'static str, toml::de::Error),
}
