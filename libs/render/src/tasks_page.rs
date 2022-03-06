use todo::Task;

use crate::{get_template_file, render_includes, Render};

pub struct TasksPage {
    pub tasks: Vec<Task>,
}

impl TasksPage {
    pub fn new(entries: Vec<Task>) -> Self {
        Self { tasks: entries }
    }
    fn render_tasks(&self) -> String {
        let mut html = String::new();
        for entry in &self.tasks {
            html.push_str(&entry.to_html());
        }
        html
    }
}

impl Render for TasksPage {
    fn render(&self) -> String {
        let mut ctx = get_template_file("tasks_page").unwrap();
        ctx = ctx.replace("<%= tasks %>", &self.render_tasks());
        render_includes(ctx, None)
    }
}
