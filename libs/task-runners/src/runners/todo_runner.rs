use std::{io::ErrorKind, str::FromStr};

use persistance::fs::utils::get_todo_location;
use render::{tasks_page::TasksPage, Render};
use serde::{Deserialize, Serialize};
use todo_list::{Task, TaskUpdate, UpdateType};
use tokio::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct NewTask {
    content: String,
}

pub struct TodoRunner {}

impl TodoRunner {
    pub async fn render() -> String {
        let todo_file_loc = get_todo_location();
        let todo_file = match tokio::fs::read_to_string(&todo_file_loc).await {
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
            .enumerate()
            .map(|(i, task)| task.to_html(Some(i)))
            .collect::<Vec<String>>();
        let ctx = TasksPage { tasks };
        ctx.render().await
    }

    pub async fn update(idx: usize, update: TaskUpdate) -> String {
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
