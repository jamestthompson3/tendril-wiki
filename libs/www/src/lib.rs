use markdown::parsers::IndexPage;
use urlencoding::encode;
use warp::{http::Uri, Filter};
use std::{
    collections::HashMap,
    sync::Arc
};

use sailfish::TemplateOnce;

use::markdown::ingestors::WebFormData;
use::markdown::ingestors::fs::write;
use::build::{RefBuilder, config::Config};
// let wiki = warp::fs::dir("public");
pub mod handlers;

use crate::handlers::*;

pub async fn server(config: Config, ref_builder: RefBuilder) {
    let wiki_location = config.wiki_location.clone();
    let indx = warp::get().and(with_user(Arc::new(config.user))).map(|user: Arc<String>| {
        let idx_ctx = IndexPage { user: user.to_string() };
        warp::reply::html(idx_ctx.render_once().unwrap())
    });
    let wiki = warp::get()
        .and(warp::path::param())
        .and(with_refs(ref_builder.clone()))
        .and(with_location(Arc::new(wiki_location)))
        .and_then(with_file);
    let nested = warp::get().and(warp::path!(String / String)).and(with_refs(ref_builder.clone())).and(with_location(Arc::new(config.wiki_location.clone()))).and_then(with_nested_file);
    let static_files = warp::path("static").and(warp::fs::dir("static"));
    let edit = warp::post()
        .and(warp::path("edit")
             .and(warp::body::content_length_limit(1024 * 32)
                  .and(warp::body::form())
                  .and(with_location(Arc::new(config.wiki_location)))
                  .map(|form_body: HashMap<String, String>, wiki_location: Arc<String>| {
                      let parsed_data = WebFormData::from(form_body);
                      let redir_uri = format!("/{}", encode(&parsed_data.title));
                      write(&wiki_location.to_string(), parsed_data).unwrap();
                      warp::redirect(redir_uri.parse::<Uri>().unwrap())
                  })));
    let routes = static_files.or(nested).or(edit).or(wiki).or(indx);
    let port: u16 = config.port;
    println!("Starting Server at: http://0.0.0.0:{}", port);
    warp::serve(routes).run(([0,0,0,0], port)).await;
}
