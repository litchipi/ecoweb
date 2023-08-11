use actix_web::http::header;
use actix_web::web::{self, Data};
use actix_web::{get, HttpResponse};

use crate::errors::raise_error;
use crate::loader::PostFilter;
use crate::render::Render;
use crate::{errors::Errcode, loader::Loader};

/// Wrapper around the response if we want to add specific headers
fn response(page: String) -> HttpResponse {
    HttpResponse::Ok().body(page)
}

// TODO    Print CSS

#[get("/")]
async fn index(ldr: Data<Loader>, rdr: Data<Render>) -> HttpResponse {
    match _index(&ldr, &rdr).await {
        Ok(r) => r,
        Err(e) => raise_error(e, &rdr),
    }
}

#[get("/rss")]
async fn get_rss_feed(ldr: Data<Loader>, rdr: Data<Render>) -> HttpResponse {
    match _get_rss_feed(&ldr, &rdr).await {
        Ok(r) => r,
        Err(e) => raise_error(e, &rdr),
    }
}
#[get("/allposts")]
async fn get_all_posts(ldr: Data<Loader>, rdr: Data<Render>) -> HttpResponse {
    match _get_all_posts(&ldr, &rdr).await {
        Ok(r) => r,
        Err(e) => raise_error(e, &rdr),
    }
}

#[get("/category/{name}")]
async fn get_category(
    args: web::Path<String>,
    ldr: Data<Loader>,
    rdr: Data<Render>,
) -> HttpResponse {
    match _get_category(args.into_inner(), &ldr, &rdr).await {
        Ok(r) => r,
        Err(e) => raise_error(e, &rdr),
    }
}

#[get("/serie/{slug}")]
async fn get_serie(slug: web::Path<String>, ldr: Data<Loader>, rdr: Data<Render>) -> HttpResponse {
    match _get_serie(slug.into_inner(), &ldr, &rdr).await {
        Ok(r) => r,
        Err(e) => raise_error(e, &rdr),
    }
}

async fn _index(ldr: &Loader, rdr: &Render) -> Result<HttpResponse, Errcode> {
    let recent_posts = ldr.posts.get_recent(PostFilter::NoFilter, true, Some(5))?;
    let all_categories = ldr.get_all_categories()?;
    let all_series = ldr.get_all_series()?;
    let rendered = rdr.render_index(recent_posts, all_categories, all_series)?;
    Ok(response(rendered))
}

async fn _get_rss_feed(ldr: &Loader, rdr: &Render) -> Result<HttpResponse, Errcode> {
    let all_posts = ldr.posts.get_recent(PostFilter::NoFilter, true, None)?;
    let rendered = rdr.render_rss_feed(all_posts)?;
    Ok(HttpResponse::Ok()
        .append_header(header::ContentType(mime::TEXT_XML))
        .body(rendered))
}

async fn _get_all_posts(ldr: &Loader, rdr: &Render) -> Result<HttpResponse, Errcode> {
    let all_posts = ldr.posts.get_recent(PostFilter::NoFilter, true, None)?;
    let rendered = rdr.render_list_allposts(all_posts)?;
    Ok(response(rendered))
}

async fn _get_category(name: String, ldr: &Loader, rdr: &Render) -> Result<HttpResponse, Errcode> {
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

    Ok(response(rendered))
}

async fn _get_serie(slug: String, ldr: &Loader, rdr: &Render) -> Result<HttpResponse, Errcode> {
    let all_posts = ldr.posts.list_posts_serie(slug.clone(), vec![])?;
    if let Some(serie_md) = ldr.get_serie_md(slug.clone())? {
        if all_posts.is_empty() {
            return Ok(HttpResponse::Ok().body(rdr.render_empty_post_list("serie")));
        }
        let mut ctxt = rdr.base_context.clone();
        ctxt.insert("filter", &serie_md.title);
        ctxt.insert("by", "serie");
        ctxt.insert("all_posts", &all_posts);
        let rendered = rdr.render_post_list(ctxt)?;
        Ok(response(rendered))
    } else {
        Err(Errcode::NotFound("serie", slug))
    }
}

async fn _get_by_tag(tag: String, ldr: &Loader, rdr: &Render) -> Result<HttpResponse, Errcode> {
    let all_posts = ldr
        .posts
        .get_recent(PostFilter::ContainsTag(tag.clone()), true, None)?;

    let mut ctxt = rdr.base_context.clone();
    ctxt.insert("filter", &tag);
    ctxt.insert("by", "Tag");
    ctxt.insert("all_posts", &all_posts);
    let rendered = rdr.render_post_list(ctxt)?;
    Ok(response(rendered))
}

// TODO    Add read time estimation inside render context (based on post content number of words)
async fn _get_post(id: u64, ldr: &Loader, rdr: &Render) -> Result<HttpResponse, Errcode> {
    let Some((post, nav)) = ldr.posts.get(id)? else {
        return Err(Errcode::NotFound("post_id", id.to_string()));
    };
    if !ldr.posts.check_rerender(&post.metadata) {
        if let Some(res) = ldr.cache.get_post_page(&post.metadata.id) {
            log::debug!("Got page from cache");
            return Ok(HttpResponse::Ok().body(res));
        }
    }

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

    let rendered = rdr.render_post(post, nav, ctxt)?;
    ldr.cache.add_post_page(id, rendered.clone());
    Ok(response(rendered))
}

#[get("/tag/{tag}")]
async fn get_by_tag(args: web::Path<String>, ldr: Data<Loader>, rdr: Data<Render>) -> HttpResponse {
    match _get_by_tag(args.into_inner(), &ldr, &rdr).await {
        Ok(r) => r,
        Err(e) => raise_error(e, &rdr),
    }
}

#[get("/post/{id}")]
async fn get_post(id: web::Path<u64>, ldr: Data<Loader>, rdr: Data<Render>) -> HttpResponse {
    match _get_post(id.into_inner(), &ldr, &rdr).await {
        Ok(r) => r,
        Err(e) => raise_error(e, &rdr),
    }
}
async fn not_found(rdr: Data<Render>) -> HttpResponse {
    HttpResponse::NotFound().body(rdr.render_not_found())
}

pub fn configure(srv: &mut web::ServiceConfig) -> Result<(), Errcode> {
    srv.service(index)
        .service(get_post)
        .service(get_all_posts)
        .service(get_serie)
        .service(get_category)
        .service(get_rss_feed)
        .service(get_by_tag)
        .default_service(web::route().to(not_found));
    Ok(())
}
