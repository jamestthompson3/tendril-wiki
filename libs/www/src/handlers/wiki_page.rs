use std::{collections::HashMap, sync::Arc};

use render::{new_page::NewPage, Render};
use tasks::CompileState;
use warp::{filters::BoxedFilter, Filter, Reply};

use crate::{
    controllers::{append, delete, edit},
    handlers::sinks::{render_file, render_nested_file},
    RefHubParts,
};

use super::{
    filters::{with_auth, with_links, with_location, with_sender},
    sinks::render_backlink_index,
    MAX_BODY_SIZE,
};

pub struct WikiPageRouter {
    pub parts: RefHubParts,
    pub wiki_location: Arc<String>,
}

impl WikiPageRouter {
    pub fn routes(&self) -> BoxedFilter<(impl Reply,)> {
        self.get_nested()
            .or(self.delete())
            .or(self.edit())
            .or(self.quick_add())
            .or(self.new_page())
            .or(self.backlink_index())
            .or(self.get())
            .boxed()
    }

    fn backlink_index(&self) -> BoxedFilter<(impl Reply,)> {
        let (links, _) = &self.parts;
        warp::get()
            .and(with_auth())
            .and(warp::path("links"))
            .and(with_links(links.to_owned()))
            .and_then(render_backlink_index)
            .boxed()
    }

    fn get(&self) -> BoxedFilter<(impl Reply,)> {
        let (links, _) = &self.parts;
        warp::get()
            .and(with_auth())
            .and(warp::path::param())
            .and(with_links(links.to_owned()))
            .and(with_location(self.wiki_location.clone()))
            .and(warp::query::<HashMap<String, String>>())
            .and_then(render_file)
            .boxed()
    }

    fn get_nested(&self) -> BoxedFilter<(impl Reply,)> {
        let (links, _) = &self.parts;
        warp::get()
            .and(with_auth())
            .and(warp::path!(String / String))
            .and(with_links(links.to_owned()))
            .and(with_location(self.wiki_location.clone()))
            .and_then(render_nested_file)
            .boxed()
    }

    fn delete(&self) -> BoxedFilter<(impl Reply,)> {
        let (_,  sender) = &self.parts;
        warp::post()
            .and(with_auth())
            .and(warp::path("delete"))
            .and(with_sender(sender.to_owned()))
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
        let (_,  sender) = &self.parts;
        warp::post()
            .and(with_auth())
            .and(
                warp::path("edit").and(
                    warp::body::content_length_limit(MAX_BODY_SIZE)
                        .and(warp::body::form())
                        .and(with_location(self.wiki_location.clone()))
                        .and(with_sender(sender.to_owned()))
                        .and(warp::query::<HashMap<String, String>>())
                        .and_then(edit),
                ),
            )
            .boxed()
    }

    fn quick_add(&self) -> BoxedFilter<(impl Reply,)> {
        let (_,  sender) = &self.parts;
        warp::post()
            .and(with_auth())
            .and(
                warp::path("quick-add").and(
                    warp::body::content_length_limit(MAX_BODY_SIZE)
                        .and(warp::body::form())
                        .and(with_location(self.wiki_location.clone()))
                        .and(with_sender(sender.to_owned()))
                        .and_then(append),
                ),
            )
            .boxed()
    }
}
