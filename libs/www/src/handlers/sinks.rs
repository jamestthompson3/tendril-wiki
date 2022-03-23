use persistance::fs::{read, ReadPageError};
use render::{link_page::LinkPage, new_page::NewPage, GlobalBacklinks, Render};
use std::collections::HashMap;

use urlencoding::decode;

pub async fn render_file(
    path: String,
    reflinks: GlobalBacklinks,
    wiki_location: String,
    query_params: HashMap<String, String>,
) -> Result<impl warp::Reply, warp::Rejection> {
    render_from_path(&wiki_location, path, reflinks, query_params)
}

pub async fn render_backlink_index(
    links: GlobalBacklinks,
) -> Result<impl warp::Reply, warp::Rejection> {
    let links = links.lock().unwrap();
    let ctx = LinkPage {
        links: links.clone(),
    };
    Ok(warp::reply::html(ctx.render()))
}

// TODO: Not repeat this the same as file
pub async fn render_nested_file(
    mut main_path: String,
    sub_path: String,
    reflinks: GlobalBacklinks,
    wiki_location: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    // I don't know why warp doesn't decode the sub path here...
    let sub_path_decoded = decode(&sub_path).unwrap();
    main_path.push_str(&sub_path_decoded);
    let page = read(&wiki_location, main_path, reflinks).map_err(|_| warp::reject())?;
    Ok(warp::reply::html(page))
}

pub fn render_from_path(
    location: &str,
    path: String,
    links: GlobalBacklinks,
    query_params: HashMap<String, String>,
) -> Result<impl warp::Reply, warp::Rejection> {
    match read(location, path.clone(), links) {
        Ok(page) => Ok(warp::reply::html(page)),
        Err(ReadPageError::PageNotFoundError) => {
            // TODO: Ideally, I want to redirect, but I'm not sure how to do this with
            // warp's filter system where some branches return HTML, and others redirect...
            let ctx = NewPage {
                title: Some(decode(&path).unwrap().into_owned()),
                linkto: query_params.get("linkto"),
                action_params: None,
            };
            Ok(warp::reply::html(ctx.render()))
        }
        _ => Err(warp::reject()),
    }
}
