use std::collections::HashMap;

use render::{bookmark_page::BookmarkAddPage, Render};
use tasks::{messages::Message, Queue};
use warp::{filters::BoxedFilter, hyper::Uri, Filter, Reply};

use super::{
    filters::{with_auth, with_queue},
    QueueHandle, MAX_BODY_SIZE,
};

pub struct BookmarkPageRouter {
    queue: QueueHandle,
}

struct Runner {}

impl Runner {
    async fn render() -> String {
        let ctx = BookmarkAddPage {};
        ctx.render().await
    }

    async fn create(form_body: HashMap<String, String>, queue: QueueHandle) -> Uri {
        let url = form_body.get("url").unwrap();
        let mut tags = form_body
            .get("tags")
            .unwrap()
            .split(',')
            .map(|s| s.to_owned())
            .collect::<Vec<String>>();
        tags.push(String::from("bookmark"));
        queue
            .push(Message::NewFromUrl {
                url: url.to_string(),
                tags,
            })
            .await
            .unwrap();
        let redir_uri = "/";
        redir_uri.parse::<Uri>().unwrap()
    }
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
                let template = Runner::render().await;
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
                let response = Runner::create(form, queue).await;
                warp::redirect(response)
            })
            .boxed()
    }
}
