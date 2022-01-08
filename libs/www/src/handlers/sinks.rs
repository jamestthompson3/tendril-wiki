use persistance::fs::{get_file_path, read, ReadPageError};
use render::{
    link_page::LinkPage, new_page::NewPage, tag_index_page::TagIndex, wiki_page::WikiPage,
    GlobalBacklinks, Render, TagMapping,
};
use std::{collections::HashMap, time::Instant};
use tasks::CompileState;

use logging::log;
use markdown::{parsers::path_to_data_structure, processors::to_template};

use urlencoding::decode;

pub async fn render_file(
    path: String,
    reflinks: (TagMapping, GlobalBacklinks),
    wiki_location: String,
    query_params: HashMap<String, String>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let (tags, links) = reflinks;
    render_from_path(&wiki_location, path, tags, links, query_params)
}

pub async fn render_backlink_index(
    links: GlobalBacklinks,
) -> Result<impl warp::Reply, warp::Rejection> {
    let now = Instant::now();
    let links = links.lock().unwrap();
    let ctx = LinkPage {
        links: links.clone(),
    };
    log(format!("[BackLinks] render: {:?}", now.elapsed()));
    Ok(warp::reply::html(ctx.render(&CompileState::Dynamic)))
}

pub async fn render_tags(tags: TagMapping) -> Result<impl warp::Reply, warp::Rejection> {
    let now = Instant::now();
    let tags = tags.lock().unwrap();
    let ctx = TagIndex { tags: tags.clone() };
    log(format!("[TagIndex] render: {:?}", now.elapsed()));
    Ok(warp::reply::html(ctx.render(&CompileState::Dynamic)))
}

pub async fn render_tag_page(
    tags: TagMapping,
    param: String,
    location: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    let now = Instant::now();
    let tags = tags.lock().unwrap();
    // I don't know why warp doesn't decode the sub path here...
    let sub_path_decoded = decode(&param).unwrap();
    // FIXME: This re-inventing the logic found in ingestors/fs.rs is a good
    // indication that the abstraction is wrong.
    match tags.get(&sub_path_decoded) {
        Some(tags) => {
            if let Ok(file_path) = get_file_path(&location, &sub_path_decoded) {
                if let Ok(note) = path_to_data_structure(&file_path) {
                    let templatted = to_template(&note);
                    let output = WikiPage::new(&templatted.page, Some(tags));
                    log(format!(
                        "[{}] render: {:?}",
                        sub_path_decoded,
                        now.elapsed()
                    ));
                    Ok(warp::reply::html(output.render(&CompileState::Dynamic)))
                } else {
                    let ctx = NewPage {
                        title: Some(sub_path_decoded),
                        linkto: None,
                        action_params: Some("?redir_to=tags"),
                    };
                    Ok(warp::reply::html(ctx.render(&CompileState::Dynamic)))
                }
            } else {
                let ctx = NewPage {
                    title: Some(sub_path_decoded),
                    linkto: None,
                    action_params: Some("?redir_to=tags"),
                };
                Ok(warp::reply::html(ctx.render(&CompileState::Dynamic)))
            }
        }
        None => Err(warp::reject()),
    }
}

// TODO: Not repeat this the same as file
pub async fn render_nested_file(
    mut main_path: String,
    sub_path: String,
    reflinks: (TagMapping, GlobalBacklinks),
    wiki_location: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    // I don't know why warp doesn't decode the sub path here...
    let sub_path_decoded = decode(&sub_path).unwrap();
    let (tags, links) = reflinks;
    main_path.push_str(sub_path_decoded.as_str());
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
            Ok(warp::reply::html(ctx.render(&CompileState::Dynamic)))
        }
        _ => Err(warp::reject()),
    }
}
