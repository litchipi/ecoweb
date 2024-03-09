#[derive(Debug)]
pub enum Errcode {
    // ContextQuery
    NoRecentPagesFound(String),

    // StorageData
    WrongStorageData(&'static str),
}
