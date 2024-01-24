use actix_web::{get, web::Data, HttpResponse};

use crate::{
    endpoints::reply,
    errors::Errcode,
    render::{Render, RenderedPage},
};

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::render::markdown::MarkdownRenderer;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct HiremeConfig {
    title: String,
    markdown_file: PathBuf,

    #[serde(default)]
    html_content: String,
}

impl HiremeConfig {
    pub fn convert_html(&mut self, root: &Path) -> Result<(), Errcode> {
        let renderer = MarkdownRenderer::init();
        let content = std::fs::read_to_string(root.join(&self.markdown_file))?;
        let (body, _) = renderer.render(content)?;
        self.html_content = body;
        Ok(())
    }
}

#[get("/hireme")]
async fn get_hireme(rdr: Data<Render>) -> HttpResponse {
    reply(render_hireme(&rdr), &rdr, None)
}

fn render_hireme(rdr: &Render) -> Result<RenderedPage, Errcode> {
    Ok(rdr.engine.read().render(&rdr.hireme_template, &rdr.base_context)?)
}