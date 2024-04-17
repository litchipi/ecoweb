use std::sync::Arc;

use actix_web::{web::Data, HttpResponse, HttpResponseBuilder};
use tera::Context;

use crate::{render::Render, storage::StorageErrorType};

#[derive(Debug)]
pub enum Errcode {
    // General
    FilesystemError(&'static str, Arc<std::io::Error>),

    // Configuration
    ConfigFileRead(Arc<std::io::Error>),
    MissingFormConfig(String, String),

    // Data extraction
    ContentIdParsing(std::num::ParseIntError),
    ParameterNotInUrl,

    // Storage
    StorageError(StorageErrorType),
    WrongStorageData(&'static str),
    ContextQueryBuild(&'static str, String),
    UnsupportedContextQuery(&'static str),

    // Render
    RegisterTemplate(String),
    MarkdownRender(mdtrans::Errcode),

    // Serialization
    TomlDecode(&'static str, toml::de::Error),
    BinaryEncode(bincode::Error),

    // External services
    Mail(crate::mail::MailErrcode),
}

impl Errcode {
    pub async fn build_http_response_from_data(
        self,
        render: Data<Render>,
        ctxt: Context,
    ) -> HttpResponse {
        self.build_http_response(render.get_ref(), ctxt).await
    }

    pub async fn build_http_response(self, render: &Render, ctxt: Context) -> HttpResponse {
        log::error!("{self:?}");
        let errpage = render.render_error(&self, ctxt).await;
        let mut b: HttpResponseBuilder = self.into();
        b.body(errpage)
    }
}

impl From<Errcode> for HttpResponseBuilder {
    fn from(val: Errcode) -> Self {
        match val {
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

// impl std::fmt::Debug for Errcode {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Errcode::TomlDecode(what, err) => {
//                 writeln!(f, "Error decoding {what}:")?;
//                 write!(f, "{}", err.to_string())
//             }
//             _ => write!(f, "{:?}", self),
//         }
//     }
// }

impl From<bincode::Error> for Errcode {
    fn from(value: bincode::Error) -> Self {
        Errcode::BinaryEncode(value)
    }
}

impl From<crate::mail::MailErrcode> for Errcode {
    fn from(value: crate::mail::MailErrcode) -> Self {
        Errcode::Mail(value)
    }
}
