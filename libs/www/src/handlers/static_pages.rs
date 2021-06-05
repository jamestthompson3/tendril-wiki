use markdown::parsers::{FileUploader, HelpPage, IndexPage, SearchPage};
use sailfish::TemplateOnce;
use warp::{Filter, Rejection, Reply};

use super::filters::{with_auth, with_user};

pub fn search_page() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::get()
        .and(with_auth())
        .and(warp::path("search"))
        .map(|| {
            let ctx = SearchPage {};
            warp::reply::html(ctx.render_once().unwrap())
        })
}

pub fn help_page() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::get()
        .and(with_auth())
        .and(warp::path("help"))
        .map(|| {
            let ctx = HelpPage {};
            warp::reply::html(ctx.render_once().unwrap())
        })
}

pub fn index(
    user: String,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(with_auth())
        .and(with_user(user))
        .map(|user: String| {
            let idx_ctx = IndexPage { user };
            warp::reply::html(idx_ctx.render_once().unwrap())
        })
}

pub fn upload_page()  -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(with_auth())
        .and(warp::path("upload"))
        .map(|| {
            let ctx = FileUploader {};
            warp::reply::html(ctx.render_once().unwrap())
        })
}
