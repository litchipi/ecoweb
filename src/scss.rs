use std::{collections::HashMap, path::PathBuf};

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

pub fn setup_css(
    root: PathBuf,
    scss: &HashMap<String, Vec<PathBuf>>,
    outdir: &PathBuf,
) -> Result<(), ScssError> {
    let grass_opts = grass::Options::default(); // TODO Get from options

    log::debug!("SCSS root: {root:?}");
    for (outname, scss_list) in scss.iter() {
        let mut out_css = String::new();
        for scss_path in scss_list.iter() {
            let fpath = root
                .join(scss_path)
                .canonicalize()
                .map_err(|e| ScssError::SourceNotFound(scss_path.clone()))?;
            out_css += grass::from_path(fpath, &grass_opts)?.as_str();
        }

        let outpath = root.join(outdir).join(outname).with_extension("css");

        #[cfg(feature = "css_minify")]
        let out_css = minifier::css::minify(&out_css)
            .map_err(|e| ScssError::MinificationError(e.to_string()))?
            .to_string();

        std::fs::write(outpath, out_css)
            .map_err(|e| ScssError::WriteCss(format!("{e:?}")))?;
    }
    Ok(())
}
