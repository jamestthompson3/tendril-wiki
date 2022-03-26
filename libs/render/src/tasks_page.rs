use todo::Task;

use crate::{get_template_file, render_includes, Render};
use async_trait::async_trait;

pub struct TasksPage {
    pub tasks: Vec<Task>,
}

impl TasksPage {
    pub fn new(entries: Vec<Task>) -> Self {
        Self { tasks: entries }
    }
    fn render_tasks(&self) -> String {
        let mut html = String::new();
        for (idx, entry) in self.tasks.iter().enumerate() {
            html.push_str(&entry.to_html(Some(idx)));
        }
        html
    }
}

#[async_trait]
impl Render for TasksPage {
    async fn render(&self) -> String {
        let mut ctx = get_template_file("tasks_page").await.unwrap();
        ctx = ctx.replace("<%= tasks %>", &self.render_tasks());
        render_includes(ctx, None).await
    }
}
