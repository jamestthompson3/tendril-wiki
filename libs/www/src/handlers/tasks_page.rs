use std::{io::ErrorKind, path::PathBuf, str::FromStr, sync::Arc};

use render::{tasks_page::TasksPage, Render};
use serde::{Deserialize, Serialize};
use todo::{Task, TaskUpdate, UpdateType};
use tokio::fs;
use warp::{filters::BoxedFilter, Filter, Reply};

use super::{
    filters::{with_auth, with_location},
    MAX_BODY_SIZE,
};

pub struct TaskPageRouter {
    pub wiki_location: Arc<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct NewTask {
    content: String,
}

// Keep all business logic here so it is easier to test and profile.
struct Runner {}

impl Runner {
    async fn render(location: String) -> String {
        let todo_file_loc = format!("{}{}", location, "todo.txt");
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
        ctx.render()
    }

    async fn update(location: String, idx: usize, update: TaskUpdate) -> String {
        let file_location = format!("{}{}", location, "todo.txt");
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

    pub async fn delete(location: String, idx: usize) -> usize {
        let file_location = format!("{}{}", location, "todo.txt");
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
    pub async fn create(location: String, new_task: NewTask) -> String {
        let parsed_todo = Task::from_str(&new_task.content).unwrap();
        let file_location = format!("{}{}", location, "todo.txt");
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
    pub fn new(wiki_location: Arc<String>) -> Self {
        Self { wiki_location }
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
            .and(with_location(self.wiki_location.clone()))
            .then(|location: String| async {
                let template: String = Runner::render(location).await;
                warp::reply::html(template)
            })
            .boxed()
    }

    fn create(&self) -> BoxedFilter<(impl Reply,)> {
        warp::post()
            .and(with_auth())
            .and(with_location(self.wiki_location.clone()))
            .and(warp::body::content_length_limit(MAX_BODY_SIZE))
            .and(warp::body::json())
            .then(|location: String, new_task: NewTask| async {
                let response = Runner::create(location, new_task).await;
                warp::reply::json(&response)
            })
            .boxed()
    }

    fn delete(&self) -> BoxedFilter<(impl Reply,)> {
        warp::delete()
            .and(with_auth())
            .and(with_location(self.wiki_location.clone()))
            .and(warp::path!("delete" / usize))
            .then(|location: String, idx: usize| async move {
                let response = Runner::delete(location, idx).await;
                warp::reply::json(&response)
            })
            .boxed()
    }

    fn update(&self) -> BoxedFilter<(impl Reply,)> {
        warp::path!("update" / usize)
            .and(with_auth())
            .and(warp::put())
            .and(with_location(self.wiki_location.clone()))
            .and(warp::body::content_length_limit(MAX_BODY_SIZE))
            .and(warp::body::json())
            .then(
                move |idx: usize, location: String, update: TaskUpdate| async move {
                    let response = Runner::update(location, idx, update).await;
                    warp::reply::json(&response)
                },
            )
            .boxed()
    }
}
