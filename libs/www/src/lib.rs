use markdown::parsers::{IndexPage, LinkPage, TagIndex, TagPage};
use urlencoding::decode;
use warp::Filter;
use std::{
    collections::HashMap,
    sync::Arc
};

use sailfish::TemplateOnce;

use::markdown::ingestors::WebFormData;
use::markdown::ingestors::fs::{write, read};
use::build::{RefBuilder, config::Config};
// let wiki = warp::fs::dir("public");

fn with_location(wiki_location: Arc<String>) -> impl Filter<Extract = (Arc<String>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || wiki_location.clone())
}

async fn with_file(path: String, refs: RefBuilder, wiki_location: Arc<String>) -> Result<impl warp::Reply, warp::Rejection> {
    match path.as_str() {
        "links" => {
            let ref_links = refs.links();
            let links = ref_links.lock().unwrap();
            let ctx = LinkPage {
                links: links.clone() 
            };
            Ok(warp::reply::html(ctx.render_once().unwrap()))
        },
        "tags" => {
            let ref_tags = refs.tags();
            let tags = ref_tags.lock().unwrap();
            let ctx = TagIndex {
                tags: tags.clone()
            };
            Ok(warp::reply::html(ctx.render_once().unwrap()))
        },
        _ => {
            let links = refs.links();
            let tags = refs.tags();
            let page = read(&wiki_location.to_string(), path, tags, links).map_err(|_| warp::reject())?;
            Ok(warp::reply::html(page))
        }
    }
}

// TODO: Not repeat this the same as file
async fn with_nested_file(mut main_path: String, sub_path: String, refs: RefBuilder, wiki_location: Arc<String>)-> Result<impl warp::Reply, warp::Rejection> {
    match main_path.as_str() {
        "tags" => {
            println!("Sub: {}", sub_path);
            let ref_tags = refs.tags();
            let tags = ref_tags.lock().unwrap();
            // I don't know why warp doesn't decode the sub path here...
            let sub_path_decoded = decode(&sub_path).unwrap();
            match tags.get(&sub_path_decoded) {
                Some(tags) => {
                    let ctx = TagPage {
                        title: sub_path_decoded,
                        tags: tags.to_owned()
                    };
                    Ok(warp::reply::html(ctx.render_once().unwrap()))
                }
                None => {
                    Err(warp::reject())
                }
            }
        },
        _ => {
            // I don't know why warp doesn't decode the sub path here...
            let sub_path_decoded = decode(&sub_path).unwrap();
            let links = refs.links();
            let tags = refs.tags();
            main_path.push_str(&sub_path_decoded.as_str());
            let page = read(&wiki_location.to_string(), main_path, tags, links).map_err(|_| warp::reject())?;
            Ok(warp::reply::html(page))
        }
    }
}

fn with_user(user: Arc<String>) -> impl Filter<Extract = (Arc<String>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || user.clone())
}

fn with_refs(refs: RefBuilder)-> impl Filter<Extract = (RefBuilder,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || refs.clone())
}
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
                      write(&wiki_location.to_string(), parsed_data).unwrap();
                      warp::reply::with_status("Ok", warp::http::StatusCode::OK)
                  })));
    let routes = static_files.or(nested).or(edit).or(wiki).or(indx);
    let port: u16 = config.port;
    println!("Starting Server at: http://0.0.0.0:{}", port);
    warp::serve(routes).run(([0,0,0,0], port)).await;
}
