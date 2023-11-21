use actix_web::http::header::{self, HeaderMap, HeaderValue};
use actix_web::web::{self, Data};
use actix_web::{get, HttpResponse};

use crate::errors::raise_error;
use crate::loader::PostFilter;
use crate::render::Render;
use crate::{errors::Errcode, loader::Loader};

/// Wrapper around the response if we want to add specific headers
fn reply(
    page: Result<String, Errcode>,
    rdr: &Render,
    add_headers: Option<HeaderMap>,
) -> HttpResponse {
    match page {
        Ok(html) => {
            let mut rep = HttpResponse::Ok();
            if let Some(ah) = add_headers {
                for header in ah {
                    rep.append_header(header);
                }
            }
            rep.body(html)
        }
        Err(e) => raise_error(e, rdr),
    }
}

async fn not_found(rdr: Data<Render>) -> HttpResponse {
    HttpResponse::NotFound().body(rdr.render_not_found())
}

#[get("/")]
async fn get_index(ldr: Data<Loader>, rdr: Data<Render>) -> HttpResponse {
    reply(index(&ldr, &rdr).await, &rdr, None)
}

async fn index(ldr: &Loader, rdr: &Render) -> Result<String, Errcode> {
    let recent_posts = ldr.posts.get_recent(PostFilter::NoFilter, true, Some(5))?;
    let all_categories = ldr.get_all_categories()?;
    let all_series = ldr.get_all_series()?;
    let rendered = rdr.render_index(recent_posts, all_categories, all_series)?;
    Ok(rendered)
}

#[get("/humans.txt")]
async fn get_humans(rdr: Data<Render>) -> HttpResponse {
    HttpResponse::Ok().append_header((header::CONTENT_TYPE, "text/plain")).body(rdr.site_context.humans_txt.clone())
}

#[get("/rss")]
async fn get_rss_feed(ldr: Data<Loader>, rdr: Data<Render>) -> HttpResponse {
    let mut add_headers = HeaderMap::new();
    add_headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(header::ContentType(mime::TEXT_XML).essence_str()).unwrap(),
    );
    reply(rss_feed(&ldr, &rdr).await, &rdr, Some(add_headers))
}

async fn rss_feed(ldr: &Loader, rdr: &Render) -> Result<String, Errcode> {
    let all_posts = ldr.posts.get_recent(PostFilter::NoFilter, true, None)?;
    let rendered = rdr.render_rss_feed(all_posts)?;
    Ok(rendered)
}

#[get("/allposts")]
async fn get_all_posts(ldr: Data<Loader>, rdr: Data<Render>) -> HttpResponse {
    reply(all_posts(&ldr, &rdr).await, &rdr, None)
}

async fn all_posts(ldr: &Loader, rdr: &Render) -> Result<String, Errcode> {
    let all_posts = ldr.posts.get_recent(PostFilter::NoFilter, true, None)?;
    let rendered = rdr.render_list_allposts(all_posts)?;
    Ok(rendered)
}

#[get("/category/{name}")]
async fn get_category(
    args: web::Path<String>,
    ldr: Data<Loader>,
    rdr: Data<Render>,
) -> HttpResponse {
    reply(
        list_by_category(args.into_inner(), &ldr, &rdr).await,
        &rdr,
        None,
    )
}

async fn list_by_category(name: String, ldr: &Loader, rdr: &Render) -> Result<String, Errcode> {
    let all_posts = ldr.posts.list_posts_category(name, vec![])?;

    let rendered = if let Some(fpost) = all_posts.get(0) {
        let category = fpost.category.clone().unwrap();
        let mut ctxt = rdr.base_context.clone();
        ctxt.insert("filter", &category);
        ctxt.insert("by", "category");
        ctxt.insert("all_posts", &all_posts);

        rdr.render_post_list(ctxt)
    } else {
        Ok(rdr.render_empty_post_list("category"))
    }?;

    Ok(rendered)
}

#[get("/serie/{slug}")]
async fn get_serie(slug: web::Path<String>, ldr: Data<Loader>, rdr: Data<Render>) -> HttpResponse {
    reply(
        list_by_serie(slug.into_inner(), &ldr, &rdr).await,
        &rdr,
        None,
    )
}

async fn list_by_serie(slug: String, ldr: &Loader, rdr: &Render) -> Result<String, Errcode> {
    let all_posts = ldr.posts.list_posts_serie(slug.clone(), vec![])?;
    if let Some(serie_md) = ldr.get_serie_md(slug.clone())? {
        if all_posts.is_empty() {
            return Ok(rdr.render_empty_post_list("serie"));
        }
        let mut ctxt = rdr.base_context.clone();
        ctxt.insert("filter", &serie_md.title);
        ctxt.insert("by", "serie");
        ctxt.insert("all_posts", &all_posts);
        let rendered = rdr.render_post_list(ctxt)?;
        Ok(rendered)
    } else {
        Err(Errcode::NotFound("serie", slug))
    }
}

#[get("/tag/{tag}")]
async fn get_by_tag(args: web::Path<String>, ldr: Data<Loader>, rdr: Data<Render>) -> HttpResponse {
    reply(list_by_tag(args.into_inner(), &ldr, &rdr).await, &rdr, None)
}

async fn list_by_tag(tag: String, ldr: &Loader, rdr: &Render) -> Result<String, Errcode> {
    let all_posts = ldr
        .posts
        .get_recent(PostFilter::ContainsTag(tag.clone()), true, None)?;

    let mut ctxt = rdr.base_context.clone();
    ctxt.insert("filter", &tag);
    ctxt.insert("by", "Tag");
    ctxt.insert("all_posts", &all_posts);
    let rendered = rdr.render_post_list(ctxt)?;
    Ok(rendered)
}

#[get("/post/{id}")]
async fn get_post(id: web::Path<u64>, ldr: Data<Loader>, rdr: Data<Render>) -> HttpResponse {
    reply(
        get_post_content(id.into_inner(), &ldr, &rdr).await,
        &rdr,
        None,
    )
}

async fn get_post_content(id: u64, ldr: &Loader, rdr: &Render) -> Result<String, Errcode> {
    let Some(post) = ldr.posts.get(id)? else {
        return Err(Errcode::NotFound("post_id", id.to_string()));
    };

    let mut ctxt = rdr.base_context.clone();
    ctxt.insert("post_metadata", &post.metadata);

    if let Some(ref slug) = post.metadata.serie {
        let serie_posts = ldr
            .posts
            .get_recent(PostFilter::Serie(slug.clone()), false, Some(0))?;
        ctxt.insert("serie_posts", &serie_posts);

        let next_in_serie = serie_posts.iter().find(|p| p.date > post.metadata.date);
        ctxt.insert("next_in_serie", &next_in_serie);

        let prev_in_serie = serie_posts
            .iter()
            .filter(|p| p.date < post.metadata.date)
            .last();
        ctxt.insert("prev_in_serie", &prev_in_serie);

        let post_serie_index = serie_posts
            .iter()
            .enumerate()
            .filter(|(_, pmd)| pmd.id == post.metadata.id)
            .last()
            .unwrap()
            .0
            + 1;
        ctxt.insert("post_serie_index", &post_serie_index);

        let serie_md = ldr.get_serie_md(slug.clone())?;
        ctxt.insert("serie_metadata", &serie_md);
    }

    if let Some(ref category) = post.metadata.category {
        let cat_posts = ldr.posts.get_recent(
            PostFilter::Combine(vec![
                PostFilter::Category(category.clone()),
                PostFilter::DifferentThan(id),
                PostFilter::NoSerie,
            ]),
            true,
            None,
        )?;
        ctxt.insert("category_posts", &cat_posts);
    }

    let rendered = rdr.render_post(post, ctxt)?;
    Ok(rendered)
}

pub fn configure(srv: &mut web::ServiceConfig) -> Result<(), Errcode> {
    srv.service(get_index)
        .service(get_post)
        .service(get_all_posts)
        .service(get_serie)
        .service(get_category)
        .service(get_rss_feed)
        .service(get_by_tag)
        .service(get_humans)
        .default_service(web::route().to(not_found));
    Ok(())
}
