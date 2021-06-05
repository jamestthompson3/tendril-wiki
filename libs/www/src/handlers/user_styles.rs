use std::fs;

use build::get_config_location;
use markdown::parsers::StylesPage;
use sailfish::TemplateOnce;
use warp::Filter;

use crate::controllers::update_styles;

use super::{filters::with_auth, MAX_BODY_SIZE};

pub fn serve_user_styles(
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("styles").and(warp::get().and(with_auth()).map(|| {
        let (path, _) = get_config_location();
        let style_location = path.join("userstyles.css");
        let body = fs::read_to_string(style_location).unwrap();
        let ctx = StylesPage { body };
        warp::reply::html(ctx.render_once().unwrap())
    }))
}

pub fn update_user_styles(
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("styles").and(
        warp::post().and(with_auth()).and(
            warp::body::content_length_limit(MAX_BODY_SIZE)
                .and(warp::body::form().and_then(update_styles)),
        ),
    )
}
