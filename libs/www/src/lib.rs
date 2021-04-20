use warp::Filter;
use std::{collections::HashMap};

use::markdown::ingestors::WebFormData;
use::markdown::ingestors::fs::write;

pub async fn server(port: u16, wiki_location: String) {
    let wiki = warp::fs::dir("public");
    let static_files = warp::path("static").and(warp::fs::dir("static"));
    let route = warp::post().and(warp::path("edit").and(warp::body::content_length_limit(1024 * 32)
        .and(warp::body::form())
        .map(move|form_body: HashMap<String, String>| {
            let parsed_data = WebFormData::from(form_body);
            write(wiki_location.clone(), parsed_data).unwrap();
            warp::reply::with_status("Ok", warp::http::StatusCode::OK)
        })));
    let routes = wiki.or(static_files).or(route);
    println!("Starting Server at: http://0.0.0.0:{}", port);
    warp::serve(routes).run(([0,0,0,0], port)).await;
}
