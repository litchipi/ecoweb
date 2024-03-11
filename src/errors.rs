#[derive(Debug)]
pub enum Errcode {
    // Configuration
    ConfigFileRead(std::io::Error),

    // ContextQuery
    NoRecentPagesFound(String),

    // StorageData
    WrongStorageData(&'static str),

    // Serialization
    TomlDecode(&'static str, toml::de::Error),
}
