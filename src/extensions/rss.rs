use crate::config::SiteConfig;
use actix_web::{get, http::header::HeaderMap, web::Data, HttpResponse};
use chrono::{DateTime, NaiveDateTime, Utc};

use crate::endpoints::reply;
use crate::errors::Errcode;
use crate::loader::{Loader, PostFilter};
use crate::post::PostMetadata;
use crate::render::Render;

#[get("/rss")]
pub async fn get_rss_feed(ldr: Data<Loader>, rdr: Data<Render>) -> HttpResponse {
    use actix_web::http::header;
    let mut add_headers = HeaderMap::new();
    add_headers.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_str(header::ContentType(mime::TEXT_XML).essence_str()).unwrap(),
    );
    reply(rss_feed(&ldr, &rdr).await, &rdr, Some(add_headers))
}

async fn rss_feed(ldr: &Loader, rdr: &Render) -> Result<String, Errcode> {
    let all_posts = ldr
        .posts
        .get_recent(PostFilter::NoFilter, true, None)
        .await?;
    let rendered = render_rss_feed(rdr, all_posts);
    Ok(rendered)
}

fn render_rss_feed(rdr: &Render, recent: Vec<PostMetadata>) -> String {
    let mut xml = "<rss version=\"2.0\"><channel>".to_string();
    to_rss_feed(&rdr.config.site_config, &mut xml);
    for post in recent {
        to_rss_item(&post, &rdr.config.site_config, &mut xml);
    }
    xml += "</channel></rss>";
    xml
}

fn to_rss_feed(cfg: &SiteConfig, xml: &mut String) {
    *xml += format!("<title>{}</title>", cfg.name).as_str();
    *xml += format!("<link>{}</link>", cfg.base_url).as_str();
    *xml += format!("<description>{}</description>", cfg.description).as_str();
    *xml += format!(
        "<managingEditor>{} ({})</managingEditor>",
        cfg.author_email, cfg.author_name,
    )
    .as_str();
    *xml += format!(
        "<webMaster>{} ({})</webMaster>",
        cfg.author_email, cfg.author_name
    )
    .as_str();
    *xml += format!("<copyright>{}</copyright>", cfg.copyrights).as_str();
}

fn to_rss_item(post: &PostMetadata, cfg: &SiteConfig, xml: &mut String) {
    *xml += "<item>";
    *xml += format!("<title>{}</title>", post.title).as_str();
    *xml += format!(
        "<link type=\"text/html\" title=\"{}\">{}/post/{}</link>",
        post.title, cfg.base_url, post.id,
    )
    .as_str();

    *xml += format!(
        "<author>{} ({})</author>",
        cfg.author_email, cfg.author_name
    )
    .as_str();
    *xml += format!("<guid isPermaLink=\"false\">{}</guid>", post.id).as_str();

    let date: DateTime<Utc> = DateTime::from_naive_utc_and_offset(
        NaiveDateTime::from_timestamp_opt(post.date, 0).unwrap(),
        Utc,
    );
    *xml += format!("<pubDate>{}</pubDate>", date.to_rfc2822()).as_str();

    if let Some(ref d) = post.description {
        *xml += format!("<description type=\"html\">{}</description>", d).as_str();
    } else {
        *xml += "<description>No description available</description>";
    }

    if let Some(ref c) = post.category {
        *xml += format!("<category>{}</category>", c).as_str();
    }
    for tag in post.tags.iter() {
        *xml += format!("<category>{}</category>", tag).as_str();
    }
    *xml += "</item>";
}
