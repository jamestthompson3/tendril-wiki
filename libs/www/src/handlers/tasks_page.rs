use std::{io::ErrorKind, str::FromStr};

use persistance::fs::utils::get_todo_location;
use render::{tasks_page::TasksPage, Render};
use serde::{Deserialize, Serialize};
use todo::{Task, TaskUpdate, UpdateType};
use tokio::fs;
use warp::{filters::BoxedFilter, Filter, Reply};

use super::{filters::with_auth, MAX_BODY_SIZE};

pub struct TaskPageRouter {}

#[derive(Debug, Serialize, Deserialize)]
struct NewTask {
    content: String,
}

// Keep all business logic here so it is easier to test and profile.
struct Runner {}

impl Runner {
    async fn render() -> String {
        let todo_file_loc = get_todo_location();
        let todo_file = match fs::read_to_string(&todo_file_loc).await {
            Ok(files) => files,
            Err(e) => match e.kind() {
                ErrorKind::NotFound => {
                    fs::File::create(todo_file_loc).await.unwrap();
                    String::with_capacity(0)
                }
                _ => {
                    panic!("Could not read todo file");
                }
            },
        };
        let tasks = todo_file
            .lines()
            .map(|l| Task::from_str(l).unwrap())
            .collect::<Vec<Task>>();
        let ctx = TasksPage { tasks };
        ctx.render().await
    }

    async fn update(idx: usize, update: TaskUpdate) -> String {
        let file_location = get_todo_location();
        let todo_file = fs::read_to_string(&file_location).await.unwrap();
        let mut tasks = todo_file.lines().collect::<Vec<&str>>();
        let mut targeted_task = Task::from_str(tasks[idx]).unwrap();
        let patch = if update.completed.is_some() {
            UpdateType::Completion(update.completed.unwrap())
        } else if update.priority.is_some() {
            UpdateType::Prio(update.priority.unwrap())
        } else if update.metadata.is_some() {
            UpdateType::Meta(update.metadata.unwrap())
        } else {
            UpdateType::Content(update.content.unwrap())
        };
        let response = targeted_task.patch(patch);
        let updated_task = targeted_task.body;
        tasks[idx] = &updated_task;
        fs::write(&file_location, tasks.join("\n")).await.unwrap();
        response.trim().into()
    }

    pub async fn delete(idx: usize) -> usize {
        let file_location = get_todo_location();
        let todo_file = fs::read_to_string(&file_location).await.unwrap();
        let tasks = todo_file
            .lines()
            .enumerate()
            .filter_map(|(file_idx, line)| {
                if file_idx != idx {
                    return Some(line);
                }
                None
            })
            .collect::<Vec<&str>>();
        fs::write(&file_location, tasks.join("\n")).await.unwrap();
        idx
    }
    pub async fn create(new_task: NewTask) -> String {
        let parsed_todo = Task::from_str(&new_task.content).unwrap();
        let file_location = get_todo_location();
        let todo_file = fs::read_to_string(&file_location).await.unwrap();
        let mut updated_todos = todo_file.lines().collect::<Vec<&str>>();
        updated_todos.insert(0, &parsed_todo.body);
        fs::write(&file_location, updated_todos.join("\n"))
            .await
            .unwrap();
        parsed_todo.to_html(Some(0))
    }
}

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
                let template: String = Runner::render().await;
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
                let response = Runner::create(new_task).await;
                warp::reply::json(&response)
            })
            .boxed()
    }

    fn delete(&self) -> BoxedFilter<(impl Reply,)> {
        warp::delete()
            .and(with_auth())
            .and(warp::path!("delete" / usize))
            .then(|idx: usize| async move {
                let response = Runner::delete(idx).await;
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
                let response = Runner::update(idx, update).await;
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
