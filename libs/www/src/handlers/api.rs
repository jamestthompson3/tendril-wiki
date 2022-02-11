use std::sync::Arc;

use warp::{Filter, Rejection, Reply};

use crate::controllers::{authorize, file, image, note_search, update_styles, unauthorize};

use super::{
    filters::{with_auth, with_location},
    MAX_BODY_SIZE,
};

pub struct APIRouter {
    pub wiki_location: Arc<String>,
    pub media_location: Arc<String>,
}

impl APIRouter {
    pub fn routes(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        self.login()
            .or(self.logout())
            .or(self.styles())
            .or(self.img())
            .or(self.files())
            .or(self.search())
    }
    fn search(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::post().and(with_auth()).and(
            warp::path("search").and(
                warp::body::content_length_limit(MAX_BODY_SIZE)
                    .and(warp::body::form())
                    .and(with_location(self.wiki_location.clone()))
                    .and_then(note_search),
            ),
        )
    }
    fn img(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::post().and(with_auth()).and(
            warp::path("files").and(
                warp::body::content_length_limit(MAX_BODY_SIZE)
                    .and(warp::header::<String>("filename"))
                    .and(warp::body::bytes())
                    .and(with_location(self.media_location.clone()))
                    .and_then(image),
            ),
        )
    }
    fn files(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::post()
            .and(with_auth())
            .and(warp::body::content_length_limit(MAX_BODY_SIZE))
            .and(warp::filters::multipart::form())
            .and(with_location(self.media_location.clone()))
            .and_then(file)
    }
    fn login(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::post().and(warp::path("login")).and(
            warp::body::content_length_limit(MAX_BODY_SIZE)
                .and(warp::body::form())
                .and_then(authorize),
        )
    }
    fn logout(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::get().and(warp::path("logout")).and_then(unauthorize)
    }
    fn styles(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path("styles").and(
            warp::post().and(with_auth()).and(
                warp::body::content_length_limit(MAX_BODY_SIZE)
                    .and(warp::body::form().and_then(update_styles)),
            ),
        )
    }
}
