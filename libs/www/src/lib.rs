use ::build::{config::Config, RefBuilder};
use ::markdown::ingestors::fs::write;
use ::markdown::ingestors::WebFormData;
use markdown::parsers::{IndexPage, NewPage};
use sailfish::TemplateOnce;
use std::{collections::HashMap, sync::Arc};
use urlencoding::encode;
use warp::{http::Uri, Filter};

pub mod handlers;

use crate::handlers::*;

pub async fn server(config: Config, ref_builder: RefBuilder) {
    let wiki_location = config.wiki_location.clone();
    let indx = warp::get()
        .and(with_user(Arc::new(config.user)))
        .map(|user: Arc<String>| {
            let idx_ctx = IndexPage {
                user: user.to_string(),
            };
            warp::reply::html(idx_ctx.render_once().unwrap())
        });
    let wiki = warp::get()
        .and(warp::path::param())
        .and(with_refs(ref_builder.clone()))
        .and(with_location(Arc::new(wiki_location)))
        .and_then(with_file);
    let nested = warp::get()
        .and(warp::path!(String / String))
        .and(with_refs(ref_builder.clone()))
        .and(with_location(Arc::new(config.wiki_location.clone())))
        .and_then(with_nested_file);
    let new_page = warp::get().and(warp::path("new").map(|| {
        let ctx = NewPage {};
        warp::reply::html(ctx.render_once().unwrap())
    }));
    let static_files = warp::path("static").and(warp::fs::dir("static"));
    let edit = warp::post().and(
        warp::path("edit").and(
            warp::body::content_length_limit(1024 * 32)
                .and(warp::body::form())
                .and(with_location(Arc::new(config.wiki_location)))
                .and(with_refs(ref_builder.clone()))
                .map(
                    |form_body: HashMap<String, String>, wiki_location: Arc<String>, mut builder: RefBuilder| {
                        let parsed_data = WebFormData::from(form_body);
                        let redir_uri = format!("/{}", encode(&parsed_data.title));
                        match write(&wiki_location.to_string(), parsed_data) {
                            Ok(()) => {
                                builder.build(&wiki_location);
                                warp::redirect(redir_uri.parse::<Uri>().unwrap())
                            }
                            Err(_) =>  {
                                warp::redirect(Uri::from_static("/error"))
                            }
                        }
                    },
                ),
        ),
    );
    let routes = static_files
        .or(nested)
        .or(edit)
        .or(new_page)
        .or(wiki)
        .or(indx);
    let port: u16 = config.port;
      println!("┌──────────────────────────────────────────────┐");
      println!("│Starting web backend @ http://127.0.0.1:{}  │", port);
      println!("└──────────────────────────────────────────────┘");
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}
