use std::sync::Arc;

use build::RefBuilder;
use warp::{Filter, Reply, filters::BoxedFilter};

use crate::controllers::{authorize, content_grammar, file, image, note_search, update_styles};

use super::{MAX_BODY_SIZE, filters::{with_auth, with_location, with_refs}};

pub struct APIRouter {
    pub wiki_location: Arc<String>,
    pub media_location: Arc<String>,
    pub reference_builder: RefBuilder
}

impl APIRouter {
    pub fn routes(&self) -> BoxedFilter<(impl Reply,)> {
        self.login()
            .or(self.styles())
            .or(self.img())
            .or(self.files())
            .or(self.search())
            .or(self.grammar())
            .boxed()
    }
    fn search(&self) -> BoxedFilter<(impl Reply,)> {
        warp::post().and(with_auth()).and(
            warp::path("search").and(
                warp::body::content_length_limit(MAX_BODY_SIZE)
                    .and(warp::body::form())
                    .and(with_location(self.wiki_location.clone()))
                    .and_then(note_search)
            )
        ).boxed()
    }
    fn img(&self) -> BoxedFilter<(impl Reply,)> {
        warp::post().and(with_auth()).and(
            warp::path("files").and(
                warp::body::content_length_limit(MAX_BODY_SIZE)
                    .and(warp::header::<String>("filename"))
                    .and(warp::body::bytes())
                    .and(with_location(self.media_location.clone()))
                    .and_then(image)
            )
        ).boxed()
    }
    fn files(&self) -> BoxedFilter<(impl Reply,)> {
        warp::post()
            .and(with_auth())
            .and(warp::body::content_length_limit(MAX_BODY_SIZE))
            .and(warp::filters::multipart::form())
            .and(with_location(self.media_location.clone()))
            .and_then(file)
            .boxed()
    }
    fn grammar(&self)-> BoxedFilter<(impl Reply,)> {
        warp::get()
            .and(with_auth())
            .and(warp::path("grammar"))
            .and(with_refs(self.reference_builder.clone()))
            .and_then(content_grammar)
            .boxed()
        }
    fn login(&self) -> BoxedFilter<(impl Reply,)> {
        warp::post().and(warp::path("login")).and(
            warp::body::content_length_limit(MAX_BODY_SIZE)
                .and(warp::body::form())
                .and_then(authorize)
        ).boxed()
    }
    fn styles(&self) -> BoxedFilter<(impl Reply,)> {
        warp::path("styles").and(
            warp::post().and(with_auth()).and(
                warp::body::content_length_limit(MAX_BODY_SIZE)
                    .and(warp::body::form().and_then(update_styles)),
            )
        ).boxed()
    }
}
