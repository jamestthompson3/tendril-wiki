use sailfish::TemplateOnce;
use std::{collections::HashMap, time::Instant};

use build::RefBuilder;
use logging::log;
use markdown::{
    ingestors::{read, ReadPageError},
    parsers::{LinkPage, NewPage, TagIndex, TagPage},
};
use urlencoding::decode;

pub async fn render_file(
    path: String,
    refs: RefBuilder,
    wiki_location: String,
    query_params: HashMap<String, String>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let now = Instant::now();
    let links = refs.links();
    let tags = refs.tags();
    match read(&wiki_location, path.clone(), tags, links) {
        Ok(page) => {
            log(format!(
                "[{}] render: {:?}",
                decode(&path).unwrap(),
                now.elapsed()
            ));
            Ok(warp::reply::html(page))
        }
        Err(ReadPageError::PageNotFoundError) => {
            // TODO: Ideally, I want to redirect, but I'm not sure how to do this with
            // warp's filter system where some branches return HTML, and others redirect...
            let ctx = NewPage {
                title: Some(decode(&path).unwrap()),
                linkto: query_params.get("linkto"),
            };

            log(format!(
                "[{}] render: {:?}",
                decode(&path).unwrap(),
                now.elapsed()
            ));
            Ok(warp::reply::html(ctx.render_once().unwrap()))
        }
        _ => Err(warp::reject()),
    }
}

pub async fn render_backlink_index(refs: RefBuilder) -> Result<impl warp::Reply, warp::Rejection> {
    let now = Instant::now();
    let ref_links = refs.links();
    let links = ref_links.lock().unwrap();
    let ctx = LinkPage {
        links: links.clone(),
    };
    log(format!("[BackLinks] render: {:?}", now.elapsed()));
    Ok(warp::reply::html(ctx.render_once().unwrap()))
}

pub async fn render_tags(refs: RefBuilder) -> Result<impl warp::Reply, warp::Rejection> {
    let now = Instant::now();
    let ref_tags = refs.tags();
    let tags = ref_tags.lock().unwrap();
    let ctx = TagIndex { tags: tags.clone() };
    log(format!("[TagIndex] render: {:?}", now.elapsed()));
    Ok(warp::reply::html(ctx.render_once().unwrap()))
}

// TODO: Not repeat this the same as file
pub async fn render_nested_file(
    mut main_path: String,
    sub_path: String,
    refs: RefBuilder,
    wiki_location: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    // I don't know why warp doesn't decode the sub path here...
    let sub_path_decoded = decode(&sub_path).unwrap();
    let links = refs.links();
    let tags = refs.tags();
    main_path.push_str(&sub_path_decoded.as_str());
    let page = read(&wiki_location, main_path, tags, links).map_err(|_| warp::reject())?;
    Ok(warp::reply::html(page))
}
