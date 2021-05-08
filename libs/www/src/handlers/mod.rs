use::markdown::ingestors::fs::read;
use tasks::normalize_wiki_location;
use warp::Filter;
use markdown::parsers::{LinkPage, TagIndex, TagPage};
use urlencoding::decode;
use std::sync::Arc
;
use sailfish::TemplateOnce;
use::build::RefBuilder;

pub fn with_location(wiki_location: String) -> impl Filter<Extract = (String,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || normalize_wiki_location(&wiki_location))
}

pub async fn with_file(path: String, refs: RefBuilder, wiki_location: String) -> Result<impl warp::Reply, warp::Rejection> {
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
            let page = read(&wiki_location, path, tags, links).map_err(|_| warp::reject())?;
            Ok(warp::reply::html(page))
        }
    }
}

// TODO: Not repeat this the same as file
pub async fn with_nested_file(mut main_path: String, sub_path: String, refs: RefBuilder, wiki_location: String)-> Result<impl warp::Reply, warp::Rejection> {
    match main_path.as_str() {
        "tags" => {
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

pub fn with_user(user: Arc<String>) -> impl Filter<Extract = (Arc<String>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || user.clone())
}

pub fn with_refs(refs: RefBuilder)-> impl Filter<Extract = (RefBuilder,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || refs.clone())
}
