use std::path::PathBuf;

#[derive(Debug)]
pub enum Errcode {
    PathDoesntExist(&'static str, PathBuf),
    Template(tera::Error),
    TemplateTypeNotBound(&'static str),
    IoError(std::io::Error),
    Scss(grass::Error),
    StringDecode(std::string::FromUtf8Error),
    JsonSerialization(serde_json::Error),
    TomlDecode(toml::de::Error),
    Syntect(syntect::Error),
    ArgumentMissing(&'static str, &'static str),
    NotFound(&'static str, String),
    MetadataValidationFailed(&'static str, &'static str),

    CopyItemsRecursive(fs_extra::error::Error),

    #[cfg(feature = "local_storage")]
    StorageError(crate::loader::storage::local_storage::LocalStorageError),
    #[cfg(feature = "css_minify")]
    CssMinifyingError(lightningcss::error::Error<lightningcss::error::MinifyErrorKind>),
    #[cfg(feature = "css_minify")]
    CssPrintingError(lightningcss::error::Error<lightningcss::error::PrinterErrorKind>),
    #[cfg(feature = "css_minify")]
    CssParsingError(String),
    #[cfg(feature = "html_minify")]
    WrongHtmlCode(minify_html_onepass::Error),
}

impl std::fmt::Display for Errcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[allow(clippy::match_single_binding)]
        match self {
            e => write!(f, "{:?}", e),
        }
    }
}

impl From<Errcode> for actix_web::error::Error {
    fn from(val: Errcode) -> Self {
        actix_web::error::ErrorInternalServerError(val)
    }
}

impl From<tera::Error> for Errcode {
    fn from(err: tera::Error) -> Self {
        Errcode::Template(err)
    }
}

impl From<std::io::Error> for Errcode {
    fn from(value: std::io::Error) -> Self {
        Errcode::IoError(value)
    }
}

impl From<Box<grass::Error>> for Errcode {
    fn from(value: Box<grass::Error>) -> Self {
        Errcode::Scss(*value)
    }
}

impl From<crate::loader::storage::local_storage::LocalStorageError> for Errcode {
    fn from(value: crate::loader::storage::local_storage::LocalStorageError) -> Self {
        Errcode::StorageError(value)
    }
}

impl From<std::string::FromUtf8Error> for Errcode {
    fn from(value: std::string::FromUtf8Error) -> Self {
        Errcode::StringDecode(value)
    }
}

impl From<serde_json::Error> for Errcode {
    fn from(value: serde_json::Error) -> Self {
        Errcode::JsonSerialization(value)
    }
}

impl From<toml::de::Error> for Errcode {
    fn from(value: toml::de::Error) -> Self {
        Errcode::TomlDecode(value)
    }
}

impl From<syntect::Error> for Errcode {
    fn from(value: syntect::Error) -> Self {
        Errcode::Syntect(value)
    }
}

#[cfg(feature = "html_minify")]
impl From<minify_html_onepass::Error> for Errcode {
    fn from(value: minify_html_onepass::Error) -> Self {
        Errcode::WrongHtmlCode(value)
    }
}

impl From<fs_extra::error::Error> for Errcode {
    fn from(value: fs_extra::error::Error) -> Self {
        Errcode::CopyItemsRecursive(value)
    }
}

#[cfg(feature = "css_minify")]
impl From<lightningcss::error::Error<lightningcss::error::MinifyErrorKind>> for Errcode {
    fn from(value: lightningcss::error::Error<lightningcss::error::MinifyErrorKind>) -> Self {
        Errcode::CssMinifyingError(value)
    }
}

#[cfg(feature = "css_minify")]
impl From<lightningcss::error::Error<lightningcss::error::PrinterErrorKind>> for Errcode {
    fn from(value: lightningcss::error::Error<lightningcss::error::PrinterErrorKind>) -> Self {
        Errcode::CssPrintingError(value)
    }
}

#[cfg(feature = "css_minify")]
impl<'a> From<lightningcss::error::Error<lightningcss::error::ParserError<'a>>> for Errcode {
    fn from(value: lightningcss::error::Error<lightningcss::error::ParserError<'a>>) -> Self {
        Errcode::CssParsingError(value.to_string())
    }
}
