use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum ScssError {
    SourceNotFound(PathBuf),
    GrassError(String),
    WriteCss(String),
    MinificationError(String),
}

impl From<Box<grass::Error>> for ScssError {
    fn from(value: Box<grass::Error>) -> Self {
        ScssError::GrassError(format!("{value:?}"))
    }
}

pub fn compile_scss(fpath: PathBuf) -> Result<String, ScssError> {
    let grass_opts = grass::Options::default(); // TODO Get from config
    let out_css = grass::from_path(fpath, &grass_opts)?;

    Ok(out_css)
}
