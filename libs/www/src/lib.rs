use warp::Filter;
use std::{
    collections::HashMap,
    sync::Arc
};


use::markdown::ingestors::WebFormData;
use::markdown::ingestors::fs::{write, read};
use::build::{RefBuilder, config::Config};
// let wiki = warp::fs::dir("public");

fn with_location(wiki_location: Arc<String>) -> impl Filter<Extract = (Arc<String>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || wiki_location.clone())
}

async fn with_file(param: String, refs: RefBuilder, wiki_location: Arc<String>) -> Result<impl warp::Reply, warp::Rejection> {
             let tags = refs.tags();
             let links = refs.links();
             let page = read(&wiki_location.to_string(), param, tags, links).map_err(|_| warp::reject())?; 
             Ok(warp::reply::html(page))
     
}

pub async fn server(config: Config, ref_builder: RefBuilder) {
    let refs = warp::any().map(move || ref_builder.clone());
    let wiki_location = config.wiki_location.clone();
    let wiki = warp::get()
        .and(warp::path::param())
        .and(refs)
        .and(with_location(Arc::new(wiki_location)))
        .and_then(with_file);
    let static_files = warp::path("static").and(warp::fs::dir("static"));
    let edit = warp::post()
        .and(warp::path("edit")
        .and(warp::body::content_length_limit(1024 * 32)
        .and(warp::body::form())
        .and(with_location(Arc::new(config.wiki_location)))
        .map(|form_body: HashMap<String, String>, wiki_location: Arc<String>| {
            let parsed_data = WebFormData::from(form_body);
             write(&wiki_location.to_string(), parsed_data).unwrap();
             warp::reply::with_status("Ok", warp::http::StatusCode::OK)
       })));
    let routes = static_files.or(wiki).or(edit);
    let port: u16 = config.port;
    println!("Starting Server at: http://0.0.0.0:{}", port);
    warp::serve(routes).run(([0,0,0,0], port)).await;
}
