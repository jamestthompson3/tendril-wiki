use std::{str::FromStr, sync::Arc};

use render::{tasks_page::TasksPage, Render};
use serde::{Deserialize, Serialize};
use tasks::CompileState;
use todo::Task;
use tokio::fs;
use warp::{filters::BoxedFilter, hyper::StatusCode, Filter, Reply};

use super::{
    filters::{with_auth, with_location},
    MAX_BODY_SIZE,
};

pub struct TaskPageRouter {
    pub wiki_location: Arc<String>,
}

// Keep all business logic here so it is easier to test and profile.
struct Runner {}
#[derive(Debug, Serialize, Deserialize)]
struct TaskUpdate {
    completed: Option<CompletionState>,
    content: Option<String>,
    priority: Option<String>,
    metadata: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CompletionState {
    done: bool,
    date: Option<String>,
}

impl Runner {
    async fn render(location: String) -> String {
        let todo_file = fs::read_to_string(format!("{}{}", location, "todo.txt"))
            .await
            .unwrap();
        let tasks = todo_file
            .lines()
            .map(|l| Task::from_str(l).unwrap())
            .collect::<Vec<Task>>();
        let ctx = TasksPage { tasks };
        ctx.render(&CompileState::Dynamic)
    }

    async fn update(idx: u32, update: TaskUpdate) {
        println!("Got Task {}'s update - {:?}", idx, update);
    }

    pub async fn delete(idx: u32) {
        println!("Deleting: {}", idx);
    }
}

impl TaskPageRouter {
    pub fn new(wiki_location: Arc<String>) -> Self {
        Self { wiki_location }
    }
    pub fn routes(&self) -> BoxedFilter<(impl Reply,)> {
        warp::any()
            .and(warp::path("tasks"))
            .and(self.delete().or(self.update()).or(self.get()))
            .boxed()
    }

    fn get(&self) -> BoxedFilter<(impl Reply,)> {
        warp::get()
            .and(with_auth())
            .and(with_location(self.wiki_location.clone()))
            .then(|location: String| async {
                let template: String = Runner::render(location).await;
                warp::reply::html(template)
            })
            .boxed()
    }

    fn delete(&self) -> BoxedFilter<(impl Reply,)> {
        warp::delete()
            .and(with_auth())
            .and(warp::path!("delete" / u32))
            .then(|idx: u32| async move {
                Runner::delete(idx).await;
                warp::reply::with_status("ok", StatusCode::OK)
            })
            .boxed()
    }

    fn update(&self) -> BoxedFilter<(impl Reply,)> {
        warp::path!("update" / u32)
            .and(with_auth())
            .and(warp::put())
            .and(warp::body::content_length_limit(MAX_BODY_SIZE))
            .and(warp::body::json())
            .then(move |idx: u32, update: TaskUpdate| async move {
                Runner::update(idx, update).await;
                warp::reply::with_status("ok", StatusCode::OK)
            })
            .boxed()
    }
}
