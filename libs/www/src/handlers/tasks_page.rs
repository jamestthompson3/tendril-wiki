use std::sync::Arc;

use warp::{filters::BoxedFilter, Filter, Reply};

use crate::controllers::{delete_tasks, edit_tasks, get_tasks};

use super::{
    filters::{with_auth, with_location},
    MAX_BODY_SIZE,
};

pub struct TaskPageRouter {
    pub wiki_location: Arc<String>,
}

impl TaskPageRouter {
    pub fn routes(&self) -> BoxedFilter<(impl Reply,)> {
        warp::any()
            .and(warp::path("tasks"))
            .and(self.delete().or(self.edit()).or(self.get())).boxed()
    }

    fn get(&self) -> BoxedFilter<(impl Reply,)> {
        warp::get()
            .and(with_auth())
            .and(with_location(self.wiki_location.clone()))
            .and_then(get_tasks)
            .boxed()
    }

    fn delete(&self) -> BoxedFilter<(impl Reply,)> {
        warp::post()
            .and(with_auth())
            .and(warp::path("delete"))
            .and(warp::body::content_length_limit(MAX_BODY_SIZE))
            .and(warp::body::form())
            .and_then(delete_tasks)
            .boxed()
    }

    fn edit(&self) -> BoxedFilter<(impl Reply,)> {
        warp::post()
            .and(with_auth())
            .and(
                warp::path("edit").and(
                    warp::body::content_length_limit(MAX_BODY_SIZE)
                        .and(warp::body::form())
                        .and_then(edit_tasks),
                ),
            )
            .boxed()
    }
}
