use std::{collections::HashMap, sync::Arc};

use build::RefBuilder;
use render::{new_page::NewPage, Render};
use tasks::CompileState;
use warp::{filters::BoxedFilter, Filter, Reply};

use crate::{
    controllers::{delete, edit, append},
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
    pub fn routes(&self) -> BoxedFilter<(impl Reply,)> {
        self.get_nested()
            .or(self.delete())
            .or(self.edit())
            .or(self.quick_add())
            .or(self.new_page())
            .or(self.tag_page())
            .or(self.tag_index())
            .or(self.backlink_index())
            .or(self.get())
            .boxed()
    }

    fn tag_index(&self) -> BoxedFilter<(impl Reply,)> {
        warp::get()
            .and(with_auth())
            .and(warp::path::path("tags"))
            .and(with_refs(self.reference_builder.clone()))
            .and_then(render_tags)
            .boxed()
    }

    fn tag_page(&self) -> BoxedFilter<(impl Reply,)> {
        warp::get()
            .and(with_auth())
            .and(warp::path::path("tags"))
            .and(with_refs(self.reference_builder.clone()))
            .and(warp::path::param())
            .and(with_location(self.wiki_location.clone()))
            .and_then(render_tag_page)
            .boxed()
    }

    fn backlink_index(&self) -> BoxedFilter<(impl Reply,)> {
        warp::get()
            .and(with_auth())
            .and(warp::path("links"))
            .and(with_refs(self.reference_builder.clone()))
            .and_then(render_backlink_index)
            .boxed()
    }

    fn get(&self) -> BoxedFilter<(impl Reply,)> {
        warp::get()
            .and(with_auth())
            .and(warp::path::param())
            .and(with_refs(self.reference_builder.clone()))
            .and(with_location(self.wiki_location.clone()))
            .and(warp::query::<HashMap<String, String>>())
            .and_then(render_file)
            .boxed()
    }

    fn get_nested(&self) -> BoxedFilter<(impl Reply,)> {
        warp::get()
            .and(with_auth())
            .and(warp::path!(String / String))
            .and(with_refs(self.reference_builder.clone()))
            .and(with_location(self.wiki_location.clone()))
            .and_then(render_nested_file)
            .boxed()
    }

    fn delete(&self) -> BoxedFilter<(impl Reply,)> {
        warp::post()
            .and(with_auth())
            .and(warp::path("delete"))
            .and(with_refs(self.reference_builder.clone()))
            .and(with_location(self.wiki_location.clone()))
            .and(warp::body::content_length_limit(MAX_BODY_SIZE))
            .and(warp::body::form())
            .and_then(delete)
            .boxed()
    }

    fn new_page(&self) -> BoxedFilter<(impl Reply,)> {
        warp::get()
            .and(with_auth())
            .and(
                warp::path("new")
                    .and(warp::query::<HashMap<String, String>>())
                    .map(|query_params: HashMap<String, String>| {
                        let ctx = NewPage {
                            title: None,
                            linkto: query_params.get("linkto"),
                            action_params: None,
                        };
                        warp::reply::html(ctx.render(&CompileState::Dynamic))
                    }),
            )
            .boxed()
    }

    fn edit(&self) -> BoxedFilter<(impl Reply,)> {
        warp::post()
            .and(with_auth())
            .and(
                warp::path("edit").and(
                    warp::body::content_length_limit(MAX_BODY_SIZE)
                        .and(warp::body::form())
                        .and(with_location(self.wiki_location.clone()))
                        .and(with_refs(self.reference_builder.clone()))
                        .and(warp::query::<HashMap<String, String>>())
                        .and_then(edit),
                ),
            )
            .boxed()
    }

    fn quick_add(&self) -> BoxedFilter<(impl Reply,)> {
        warp::post()
            .and(with_auth())
            .and(
                warp::path("quick-add").and(
                    warp::body::content_length_limit(MAX_BODY_SIZE)
                        .and(warp::body::form())
                        .and(with_location(self.wiki_location.clone()))
                        .and_then(append),
                ),
            )
            .boxed()
    }
}
