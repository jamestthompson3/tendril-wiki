use std::{collections::HashMap, sync::Arc};

use build::RefBuilder;
use markdown::parsers::NewPage;
use sailfish::TemplateOnce;
use warp::{Filter, Rejection, Reply};

use crate::{
    controllers::{delete, edit},
    handlers::filters::with_nested_file,
};

use super::{
    filters::{with_auth, with_file, with_location, with_refs},
    MAX_BODY_SIZE,
};

pub struct WikiPageRouter {
    pub reference_builder: RefBuilder,
    pub wiki_location: Arc<String>,
}

impl WikiPageRouter {
    pub fn routes(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        self.get_nested()
            .or(self.delete())
            .or(self.edit())
            .or(self.new_page())
            .or(self.get())
    }

    pub fn get(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::get()
            .and(with_auth())
            .and(warp::path::param())
            .and(with_refs(self.reference_builder.clone()))
            .and(with_location(self.wiki_location.clone()))
            .and(warp::query::<HashMap<String, String>>())
            .and_then(with_file)
    }

    pub fn get_nested(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::get()
            .and(with_auth())
            .and(warp::path!(String / String))
            .and(with_refs(self.reference_builder.clone()))
            .and(with_location(self.wiki_location.clone()))
            .and_then(with_nested_file)
    }

    pub fn delete(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::post()
            .and(with_auth())
            .and(warp::path("delete"))
            .and(with_refs(self.reference_builder.clone()))
            .and(with_location(self.wiki_location.clone()))
            .and(warp::body::content_length_limit(MAX_BODY_SIZE))
            .and(warp::body::form())
            .and_then(delete)
    }

    pub fn new_page(
        &self,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::get().and(with_auth()).and(
            warp::path("new")
                .and(warp::query::<HashMap<String, String>>())
                .map(|query_params: HashMap<String, String>| {
                    let ctx = NewPage {
                        title: None,
                        linkto: query_params.get("linkto"),
                    };
                    warp::reply::html(ctx.render_once().unwrap())
                }),
        )
    }

    pub fn edit(&self) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::post().and(with_auth()).and(
            warp::path("edit").and(
                warp::body::content_length_limit(MAX_BODY_SIZE)
                    .and(warp::body::form())
                    .and(with_location(self.wiki_location.clone()))
                    .and(with_refs(self.reference_builder.clone()))
                    .and_then(edit),
            ),
        )
    }
}
