use std::{collections::HashMap, sync::Arc};

use build::RefBuilder;
use markdown::parsers::NewPage;
use sailfish::TemplateOnce;
use warp::{Filter, Rejection, Reply};

use crate::{
    controllers::{delete, edit},
    handlers::sinks::{render_file, render_nested_file},
};

use super::{
    filters::{with_auth, with_location, with_refs},
    sinks::{render_backlink_index, render_tag_page, render_tags},
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
            .or(self.tag_page())
            .or(self.tag_index())
            .or(self.backlink_index())
            .or(self.get())
    }

    fn tag_index(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::get()
            .and(with_auth())
            .and(warp::path::path("tags"))
            .and(with_refs(self.reference_builder.clone()))
            .and_then(render_tags)
    }

    fn tag_page(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::get()
            .and(with_auth())
            .and(warp::path::path("tags"))
            .and(with_refs(self.reference_builder.clone()))
            .and(warp::path::param())
            .and_then(render_tag_page)
    }

    fn backlink_index(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::get()
            .and(with_auth())
            .and(warp::path("links"))
            .and(with_refs(self.reference_builder.clone()))
            .and_then(render_backlink_index)
    }

    fn get(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::get()
            .and(with_auth())
            .and(warp::path::param())
            .and(with_refs(self.reference_builder.clone()))
            .and(with_location(self.wiki_location.clone()))
            .and(warp::query::<HashMap<String, String>>())
            .and_then(render_file)
    }

    fn get_nested(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::get()
            .and(with_auth())
            .and(warp::path!(String / String))
            .and(with_refs(self.reference_builder.clone()))
            .and(with_location(self.wiki_location.clone()))
            .and_then(render_nested_file)
    }

    fn delete(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::post()
            .and(with_auth())
            .and(warp::path("delete"))
            .and(with_refs(self.reference_builder.clone()))
            .and(with_location(self.wiki_location.clone()))
            .and(warp::body::content_length_limit(MAX_BODY_SIZE))
            .and(warp::body::form())
            .and_then(delete)
    }

    fn new_page(&self) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
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

    fn edit(&self) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
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
