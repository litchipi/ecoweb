use std::path::PathBuf;

use actix_web::HttpResponse;

use crate::render::Render;

use thiserror::Error;

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum Errcode {
    #[error("Path doesn't exist")]
    PathDoesntExist(&'static str, PathBuf),

    #[error("Error while creating HTML from template")]
    Template(#[from] tera::Error),

    #[error("Template type is not bound")]
    TemplateTypeNotBound(&'static str),

    #[error("Input / Output error")]
    IoError(#[from] std::io::Error),

    #[error("Error while compiling SCSS to CSS")]
    Scss(#[from] Box<grass::Error>),

    #[error("Error while decoding string from UTF-8")]
    StringDecode(#[from] std::string::FromUtf8Error),

    #[error("Error while (de)serializing to JSON")]
    JsonSerialization(#[from] serde_json::Error),

    #[error("Error while decoding to TOML")]
    TomlDecode(#[from] toml::de::Error),

    #[error("Error while generating syntax coloring")]
    Syntect(#[from] syntect::Error),

    #[error("Not found")]
    NotFound(&'static str, String),

    #[error("Copy items failed")]
    CopyItemsRecursive(#[from] fs_extra::error::Error),

    #[cfg(feature = "local_storage")]
    #[error("Local storage error")]
    StorageError(#[from] crate::loader::storage::local_storage::LocalStorageError),

    #[error("Cannot perform HTTP request")]
    FailedHttpRequest(#[from] awc::error::SendRequestError),

    #[error("Cannot get HTTP reply body")]
    RequestPayloadError(#[from] awc::error::PayloadError),

    #[error("Cannot read ZIP file")]
    ZipRead(#[from] zip::result::ZipError),

    #[error("Got error status code")]
    ErrorStatusCode(actix_web::http::StatusCode),

    #[error("An error occured while converting Markdown to HTML")]
    MdTransError(#[from] mdtrans::Errcode),

    #[cfg(feature = "css_minify")]
    #[error("Error while minifying CSS")]
    CssMinifyingError(#[from] lightningcss::error::Error<lightningcss::error::MinifyErrorKind>),

    #[cfg(feature = "css_minify")]
    #[error("Error while printing CSS")]
    CssPrintingError(#[from] lightningcss::error::Error<lightningcss::error::PrinterErrorKind>),

    #[cfg(feature = "css_minify")]
    #[error("Error while parsing CSS")]
    CssParsingError(String),

    #[cfg(feature = "html_minify")]
    #[error("Error while minifying HTML")]
    BadHtmlCode(minify_html_onepass::Error),
}

#[cfg(feature = "css_minify")]
impl<'a> From<lightningcss::error::Error<lightningcss::error::ParserError<'a>>> for Errcode {
    fn from(value: lightningcss::error::Error<lightningcss::error::ParserError>) -> Self {
        Errcode::CssParsingError(value.to_string())
    }
}

#[cfg(feature = "html_minify")]
impl From<minify_html_onepass::Error> for Errcode {
    fn from(value: minify_html_onepass::Error) -> Self {
        Errcode::BadHtmlCode(value)
    }
}

pub fn raise_error(err: Errcode, render: &Render) -> HttpResponse {
    // TODO    Handle more error cases
    let errstr = match &err {
        Errcode::NotFound(content_type, content_id) => {
            format!("Unable to find {content_type} of ID {content_id}")
        }
        // Errcode::PathDoesntExist(_, _) => todo!(),
        // Errcode::Template(_) => todo!(),
        // Errcode::TemplateTypeNotBound(_) => todo!(),
        // Errcode::IoError(_) => todo!(),
        // Errcode::Scss(_) => todo!(),
        // Errcode::StringDecode(_) => todo!(),
        // Errcode::JsonSerialization(_) => todo!(),
        // Errcode::TomlDecode(_) => todo!(),
        // Errcode::Syntect(_) => todo!(),
        // Errcode::MetadataValidationFailed(_, _) => todo!(),
        // Errcode::CopyItemsRecursive(_) => todo!(),
        // Errcode::StorageError(err) => err.get_err_str(),
        // Errcode::CssMinifyingError(_) => todo!(),
        // Errcode::CssPrintingError(_) => todo!(),
        // Errcode::CssParsingError(_) => todo!(),
        #[cfg(feature = "html_minify")]
        Errcode::BadHtmlCode(e) => format!("Bad HTML code: {e:?}"),
        e => format!("{e:?}"),
    };
    // TODO    Handle more error cases
    let mut builder = HttpResponse::InternalServerError();
    builder.body(render.render_error(errstr))
}
