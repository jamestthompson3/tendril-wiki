use std::collections::HashMap;

use render::{bookmark_page::BookmarkAddPage, Render};
use task_runners::{runners::bookmark_runner::BookmarkRunner, QueueHandle};
use warp::{filters::BoxedFilter, hyper::Uri, Filter, Reply};

use super::{
    filters::{with_auth, with_queue},
    MAX_BODY_SIZE,
};

pub struct BookmarkPageRouter {
    queue: QueueHandle,
}

impl BookmarkPageRouter {
    pub fn new(queue: QueueHandle) -> Self {
        Self { queue }
    }
    pub fn routes(&self) -> BoxedFilter<(impl Reply,)> {
        warp::any()
            .and(warp::path("new_bookmark"))
            .and(self.get().or(self.post()))
            .boxed()
    }
    fn get(&self) -> BoxedFilter<(impl Reply,)> {
        warp::get()
            .and(with_auth())
            .then(|| async {
                let ctx = BookmarkAddPage {};
                let template = ctx.render().await;
                warp::reply::html(template)
            })
            .boxed()
    }
    fn post(&self) -> BoxedFilter<(impl Reply,)> {
        warp::post()
            .and(with_auth())
            .and(warp::body::content_length_limit(MAX_BODY_SIZE).and(warp::body::form()))
            .and(with_queue(self.queue.to_owned()))
            .then(|form: HashMap<String, String>, queue: QueueHandle| async {
                let next_page = BookmarkRunner::create(form, queue).await;
                let response = next_page.parse::<Uri>().unwrap();
                warp::redirect(response)
            })
            .boxed()
    }
}
