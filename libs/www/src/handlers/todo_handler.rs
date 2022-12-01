use task_runners::runners::todo_runner::{NewTask, TodoRunner};
use todo_list::TaskUpdate;
use warp::{filters::BoxedFilter, Filter, Reply};

use super::{filters::with_auth, MAX_BODY_SIZE};

pub struct TaskPageRouter {}

impl TaskPageRouter {
    pub fn new() -> Self {
        Self {}
    }
    pub fn routes(&self) -> BoxedFilter<(impl Reply,)> {
        warp::any()
            .and(warp::path("tasks"))
            .and(
                self.delete()
                    .or(self.update())
                    .or(self.create())
                    .or(self.get()),
            )
            .boxed()
    }

    fn get(&self) -> BoxedFilter<(impl Reply,)> {
        warp::get()
            .and(with_auth())
            .then(|| async {
                let template: String = TodoRunner::render().await;
                warp::reply::html(template)
            })
            .boxed()
    }

    fn create(&self) -> BoxedFilter<(impl Reply,)> {
        warp::post()
            .and(with_auth())
            .and(warp::body::content_length_limit(MAX_BODY_SIZE))
            .and(warp::body::json())
            .then(|new_task: NewTask| async {
                let response = TodoRunner::create(new_task).await;
                warp::reply::json(&response)
            })
            .boxed()
    }

    fn delete(&self) -> BoxedFilter<(impl Reply,)> {
        warp::delete()
            .and(with_auth())
            .and(warp::path!("delete" / usize))
            .then(|idx: usize| async move {
                let response = TodoRunner::delete(idx).await;
                warp::reply::json(&response)
            })
            .boxed()
    }

    fn update(&self) -> BoxedFilter<(impl Reply,)> {
        warp::path!("update" / usize)
            .and(with_auth())
            .and(warp::put())
            .and(warp::body::content_length_limit(MAX_BODY_SIZE))
            .and(warp::body::json())
            .then(move |idx: usize, update: TaskUpdate| async move {
                let response = TodoRunner::update(idx, update).await;
                warp::reply::json(&response)
            })
            .boxed()
    }
}

impl Default for TaskPageRouter {
    fn default() -> Self {
        Self::new()
    }
}
