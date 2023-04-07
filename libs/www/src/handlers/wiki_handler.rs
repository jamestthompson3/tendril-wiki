use std::collections::HashMap;

use task_runners::{runners::wiki_runner::WikiRunner, QueueHandle};
use urlencoding::decode;
use warp::{filters::BoxedFilter, hyper::Uri, Filter, Reply};
use wikitext::{GlobalBacklinks, PatchData};

use crate::RefHubParts;

use super::{
    filters::{reply_on_result, with_auth, with_links, with_queue},
    MAX_BODY_SIZE,
};

pub struct WikiPageRouter {
    parts: RefHubParts,
}

impl WikiPageRouter {
    pub fn new(parts: RefHubParts) -> Self {
        Self { parts }
    }
    pub fn routes(&self) -> BoxedFilter<(impl Reply,)> {
        self.get_nested()
            .or(self.delete())
            .or(self.edit())
            .or(self.quick_add())
            .or(self.new_page())
            .or(self.get())
            .boxed()
    }

    fn get(&self) -> BoxedFilter<(impl Reply,)> {
        let (links, _) = &self.parts;
        warp::get()
            .and(with_auth())
            .and(warp::path::param())
            .and(with_links(links.clone()))
            .and(warp::query::<HashMap<String, String>>())
            .then(
                |path: String,
                 reflinks: GlobalBacklinks,
                 query_params: HashMap<String, String>| async move {
                    let links = reflinks.lock().await;
                    let links = links.get(&*path);
                    let runner = WikiRunner {};
                    let response = runner.render_file(path, links, query_params).await;
                    warp::reply::html(response)
                },
            )
            .boxed()
    }

    fn get_nested(&self) -> BoxedFilter<(impl Reply,)> {
        let (links, _) = &self.parts;
        warp::get()
            .and(with_auth())
            .and(warp::path!(String / String))
            .and(with_links(links.to_owned()))
            .then(
                |main_path: String,
                 sub_path: String,
                 reflinks: GlobalBacklinks| async move {
                    let runner = WikiRunner {};
                    let main_path = decode(&main_path).unwrap().to_string();
                    let sub_path = decode(&sub_path).unwrap().to_string();
                    let links = reflinks.lock().await;
                    let links = links.get(&*sub_path);
                    let response = runner.render_nested_file(main_path, sub_path, links).await;
                    warp::reply::html(response.unwrap())
                },
            )
            .boxed()
    }

    fn delete(&self) -> BoxedFilter<(impl Reply,)> {
        let (_, queue) = &self.parts;
        warp::post()
            .and(with_auth())
            .and(warp::path("delete"))
            .and(with_queue(queue.to_owned()))
            .and(warp::body::content_length_limit(MAX_BODY_SIZE))
            .and(warp::body::form())
            .then(
                |queue: QueueHandle, form_body: HashMap<String, String>| async {
                    let response = WikiRunner::delete(queue, form_body).await;
                    let response = response.parse::<Uri>().unwrap();
                    warp::redirect(response)
                },
            )
            .boxed()
    }

    fn new_page(&self) -> BoxedFilter<(impl Reply,)> {
        warp::get()
            .and(with_auth())
            .and(
                warp::path("new")
                    .and(warp::query::<HashMap<String, String>>())
                    .then(|query_params: HashMap<String, String>| async {
                        let response = WikiRunner::render_new(query_params).await;
                        warp::reply::html(response)
                    }),
            )
            .boxed()
    }

    fn edit(&self) -> BoxedFilter<(impl Reply,)> {
        let (_, queue) = &self.parts;
        warp::post()
            .and(with_auth())
            .and(
                warp::path("edit").and(
                    warp::body::content_length_limit(MAX_BODY_SIZE)
                        .and(warp::body::json())
                        .and(with_queue(queue.to_owned()))
                        .then(|body: PatchData, queue: QueueHandle| async {
                            reply_on_result(WikiRunner::edit(body, queue).await)
                        }),
                ),
            )
            .boxed()
    }

    fn quick_add(&self) -> BoxedFilter<(impl Reply,)> {
        let (_, queue) = &self.parts;
        warp::post()
            .and(with_auth())
            .and(
                warp::path("quick-add").and(
                    warp::body::content_length_limit(MAX_BODY_SIZE)
                        .and(warp::body::json())
                        .and(with_queue(queue.to_owned()))
                        .then(|body: PatchData, queue: QueueHandle| async {
                            reply_on_result(WikiRunner::append(body, queue).await)
                        }),
                ),
            )
            .boxed()
    }
}
