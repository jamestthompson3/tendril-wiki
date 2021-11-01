use persistance::fs::{get_file_path, read, ReadPageError};
use render::{
    link_page::LinkPage, new_page::NewPage, tag_index_page::TagIndex, wiki_page::WikiPage, Render,
};
use std::{collections::HashMap, time::Instant};

use build::RefBuilder;
use logging::log;
use markdown::{
    parsers::{path_to_data_structure, GlobalBacklinks, TagMapping},
    processors::to_template,
};

use urlencoding::decode;

pub async fn render_file(
    path: String,
    refs: RefBuilder,
    wiki_location: String,
    query_params: HashMap<String, String>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let links = refs.links();
    let tags = refs.tags();
    render_from_path(&wiki_location, path, tags, links, query_params)
}

pub async fn render_backlink_index(refs: RefBuilder) -> Result<impl warp::Reply, warp::Rejection> {
    let now = Instant::now();
    let ref_links = refs.links();
    let links = ref_links.lock().unwrap();
    let ctx = LinkPage {
        links: links.clone(),
    };
    log(format!("[BackLinks] render: {:?}", now.elapsed()));
    Ok(warp::reply::html(ctx.render()))
}

pub async fn render_tags(refs: RefBuilder) -> Result<impl warp::Reply, warp::Rejection> {
    let now = Instant::now();
    let ref_tags = refs.tags();
    let tags = ref_tags.lock().unwrap();
    let ctx = TagIndex { tags: tags.clone() };
    log(format!("[TagIndex] render: {:?}", now.elapsed()));
    Ok(warp::reply::html(ctx.render()))
}

pub async fn render_tag_page(
    refs: RefBuilder,
    param: String,
    location: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    let now = Instant::now();
    let ref_tags = refs.tags();
    let tags = ref_tags.lock().unwrap();
    // I don't know why warp doesn't decode the sub path here...
    let sub_path_decoded = decode(&param).unwrap();
    // FIXME: This re-inventing the logic found in ingestors/fs.rs is a good
    // indication that the abstraction is wrong.
    match tags.get(&sub_path_decoded) {
        Some(tags) => {
            if let Ok(file_path) = get_file_path(&location, &sub_path_decoded) {
                if let Ok(note) = path_to_data_structure(&file_path) {
                    let templatted = to_template(&note);
                    let output = WikiPage::new(&templatted.page, Some(tags), false);
                    log(format!(
                        "[{}] render: {:?}",
                        sub_path_decoded,
                        now.elapsed()
                    ));
                    Ok(warp::reply::html(output.render()))
                } else {
                    let ctx = NewPage {
                        title: Some(sub_path_decoded),
                        linkto: None,
                        action_params: Some("?redir_to=tags"),
                    };
                    Ok(warp::reply::html(ctx.render()))
                }
            } else {
                let ctx = NewPage {
                    title: Some(sub_path_decoded),
                    linkto: None,
                    action_params: Some("?redir_to=tags"),
                };
                Ok(warp::reply::html(ctx.render()))
            }
        }
        None => Err(warp::reject()),
    }
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

pub fn render_from_path(
    location: &str,
    path: String,
    tags: TagMapping,
    links: GlobalBacklinks,
    query_params: HashMap<String, String>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let now = Instant::now();
    match read(location, path.clone(), tags, links) {
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
                action_params: None,
            };

            log(format!(
                "[{}] render: {:?}",
                decode(&path).unwrap(),
                now.elapsed()
            ));
            Ok(warp::reply::html(ctx.render()))
        }
        _ => Err(warp::reject()),
    }
}
