use actix_web::http::header::HeaderMap;
use actix_web::web::{Data, Path, ServiceConfig};
use actix_web::{get, HttpResponse};
use paste::paste;

use crate::errors::raise_error;
use crate::loader::PostFilter;
use crate::render::Render;
use crate::{errors::Errcode, loader::Loader};

// Generate wrapper functions allowing to call the `reply` function every time
// Also automatically get the loader and render structs, and get inner args from path
macro_rules! endpoint {
    ($($name:ident: $endpoint:expr => ($($arg:ident: $type:ty),*));+ $(;)?) => {
        $(
            paste! {
                #[get($endpoint)]
                async fn [<$name _endpoint>]($($arg: Path<$type>,)* ldr: Data<Loader>, rdr: Data<Render>) -> HttpResponse {
                    reply($name($($arg.into_inner(),)* &ldr, &rdr).await, &rdr, None)
                }
            }
        )+

        async fn not_found(rdr: Data<Render>) -> HttpResponse {
            HttpResponse::NotFound().body(rdr.render_not_found())
        }

        pub fn configure_all_endpoints(srv: &mut ServiceConfig) {
            $(
                paste!{
                    srv.service([<$name _endpoint>]);
                }
            )+
            srv.default_service(actix_web::web::route().to(not_found));
        }
    };
}

// Define all the endpoints, using the macro defined above
endpoint!(
    index: "/" => ();
    all_posts: "/allposts" => ();
    list_by_category: "/category/{name}" => (name: String);
    list_by_serie: "/serie/{slug}" => (slug: String);
    list_by_tag: "/tag/{tag}" => (tag: String);
    get_post: "/post/{id}" => (id: u64);
);

// Wrapper around the response
// Displays a nice error if we get an Errcode
// Is able to add headers also if needed
pub fn reply(
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

// ENDPOINT FUNCTIONS

async fn index(ldr: &Loader, rdr: &Render) -> Result<String, Errcode> {
    let recent_posts = ldr
        .posts
        .get_recent(PostFilter::NoFilter, true, Some(5))
        .await?;
    let all_categories = ldr.get_all_categories().await?;
    let all_series = ldr.get_all_series().await?;
    let rendered = rdr.render_index(recent_posts, all_categories, all_series)?;
    Ok(rendered)
}

async fn all_posts(ldr: &Loader, rdr: &Render) -> Result<String, Errcode> {
    let all_posts = ldr
        .posts
        .get_recent(PostFilter::NoFilter, true, None)
        .await?;
    let rendered = rdr.render_list_allposts(all_posts)?;
    Ok(rendered)
}

async fn list_by_category(name: String, ldr: &Loader, rdr: &Render) -> Result<String, Errcode> {
    let all_posts = ldr.posts.list_posts_category(name, vec![]).await?;

    let rendered = if let Some(fpost) = all_posts.first() {
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

async fn list_by_serie(slug: String, ldr: &Loader, rdr: &Render) -> Result<String, Errcode> {
    let all_posts = ldr.posts.list_posts_serie(slug.clone(), vec![]).await?;
    if let Some(serie_md) = ldr.get_serie_md(slug.clone()).await? {
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

async fn list_by_tag(tag: String, ldr: &Loader, rdr: &Render) -> Result<String, Errcode> {
    let all_posts = ldr
        .posts
        .get_recent(PostFilter::ContainsTag(tag.clone()), true, None)
        .await?;

    let mut ctxt = rdr.base_context.clone();
    ctxt.insert("filter", &tag);
    ctxt.insert("by", "Tag");
    ctxt.insert("all_posts", &all_posts);
    let rendered = rdr.render_post_list(ctxt)?;
    Ok(rendered)
}

async fn get_post(id: u64, ldr: &Loader, rdr: &Render) -> Result<String, Errcode> {
    let Some(post) = ldr.posts.get(id).await? else {
        return Err(Errcode::NotFound("post_id", id.to_string()));
    };

    let mut ctxt = rdr.base_context.clone();
    ctxt.insert("post_metadata", &post.metadata);

    if let Some(ref slug) = post.metadata.serie {
        let serie_posts = ldr
            .posts
            .get_recent(PostFilter::Serie(slug.clone()), false, Some(0))
            .await?;
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

        let serie_md = ldr.get_serie_md(slug.clone()).await?;
        ctxt.insert("serie_metadata", &serie_md);
    }

    if let Some(ref category) = post.metadata.category {
        let cat_posts = ldr
            .posts
            .get_recent(
                PostFilter::Combine(vec![
                    PostFilter::Category(category.clone()),
                    PostFilter::DifferentThan(id),
                    PostFilter::NoSerie,
                ]),
                true,
                None,
            )
            .await?;
        ctxt.insert("category_posts", &cat_posts);
    }

    let rendered = rdr.render_post(post, ctxt)?;
    Ok(rendered)
}
