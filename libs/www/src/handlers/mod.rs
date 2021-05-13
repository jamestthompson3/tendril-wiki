pub mod filters;

pub use self::filters::*;

use build::RefBuilder;
use markdown::parsers::{
    IndexPage, NewPage, SearchPage, SearchResultsContextPage, SearchResultsPage,
};
use sailfish::TemplateOnce;
use std::{collections::HashMap, sync::Arc};
use urlencoding::encode;

use markdown::ingestors::fs::write;
use markdown::ingestors::EditPageData;

use tasks::{context_search, search};
use warp::{http::Uri, Filter};

pub fn index(
    user: String,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get().and(with_user(user)).map(|user: String| {
        let idx_ctx = IndexPage { user };
        warp::reply::html(idx_ctx.render_once().unwrap())
    })
}

pub fn wiki(
    ref_builder: RefBuilder,
    location: Arc<String>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path::param())
        .and(with_refs(ref_builder))
        .and(with_location(location))
        .and_then(with_file)
}

pub fn nested_file(
    ref_builder: RefBuilder,
    location: Arc<String>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path!(String / String))
        .and(with_refs(ref_builder.clone()))
        .and(with_location(location))
        .and_then(with_nested_file)
}

pub fn new_page() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get().and(warp::path("new").map(|| {
        let ctx = NewPage { title: None };
        warp::reply::html(ctx.render_once().unwrap())
    }))
}

pub fn search_handler(
    location: Arc<String>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post().and(
        warp::path("search").and(
            warp::body::content_length_limit(1024 * 32)
                .and(warp::body::form())
                .and(with_location(location))
                .map(
                    |form_body: HashMap<String, String>, wiki_location: String| {
                        let term = form_body.get("term").unwrap();
                        let include_context = form_body.get("context");
                        match include_context {
                            Some(_) => {
                                let found_pages = context_search(term, &wiki_location);
                                // TODO: Maybe not a separate page here?
                                let ctx = SearchResultsContextPage { pages: found_pages };
                                warp::reply::html(ctx.render_once().unwrap())
                            }
                            None => {
                                let found_pages = search(term, &wiki_location);
                                let ctx = SearchResultsPage { pages: found_pages };
                                warp::reply::html(ctx.render_once().unwrap())
                            }
                        }
                    },
                ),
        ),
    )
}

pub fn edit_handler(
    ref_builder: RefBuilder,
    location: Arc<String>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post().and(
        warp::path("edit").and(
            warp::body::content_length_limit(1024 * 32)
                .and(warp::body::form())
                .and(with_location(location))
                .and(with_refs(ref_builder))
                .map(
                    |form_body: HashMap<String, String>,
                     wiki_location: String,
                     mut builder: RefBuilder| {
                        let parsed_data = EditPageData::from(form_body);
                        let redir_uri = format!("/{}", encode(&parsed_data.title));
                        match write(&wiki_location, parsed_data, builder.links()) {
                            Ok(()) => {
                                builder.build(&wiki_location);
                                warp::redirect(redir_uri.parse::<Uri>().unwrap())
                            }
                            Err(e) => {
                                eprintln!("{}", e);
                                warp::redirect(Uri::from_static("/error"))
                            }
                        }
                    },
                ),
        ),
    )
}

pub fn search_page() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get().and(warp::path("search")).map(|| {
        let ctx = SearchPage {};
        warp::reply::html(ctx.render_once().unwrap())
    })
}
