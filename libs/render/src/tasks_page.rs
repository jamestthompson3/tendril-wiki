use std::collections::HashMap;

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
        let mut entry_list = String::new();
        for task in &self.tasks {
            let status = self.format_status(&task.completed);
            let priority = task
                .priority
                .to_owned()
                .unwrap_or_else(|| String::with_capacity(0));
            let created = task
                .created
                .to_owned()
                .unwrap_or_else(|| String::with_capacity(0));
            let project = self.format_contextual_data(&task.project);
            let context = self.format_contextual_data(&task.context);
            let metadata = self.format_metadata(&task.metadata);
            let table_html = format!(
                r#"<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>"#,
                status, created, priority, task.body, project, context, metadata
            );
            entry_list.push_str(&table_html);
        }
        entry_list
    }
    fn format_status(&self, completed: &(bool, Option<String>)) -> String {
        let (done, date) = completed;
        if let true = done {
            if date.is_some() {
                format!("✅ {}", date.as_ref().unwrap())
            } else {
                "✅".into()
            }
        } else {
            String::with_capacity(0)
        }
    }
    fn format_contextual_data(&self, context: &[String]) -> String {
        context
            .iter()
            .fold(String::new(), |mut formatted_str, ctx| {
                let ctx_string = format!("<a href=\"{}\" target=\"_blank\">{}</a>", ctx, ctx);
                formatted_str.push_str(&ctx_string);
                formatted_str
            })
    }
    fn format_metadata(&self, metadata: &HashMap<String, String>) -> String {
        metadata
            .iter()
            .fold(String::new(), |mut formatted_str, (key, value)| {
                let ctx_string = format!("<strong>{}:</strong> {}", key, value);
                formatted_str.push_str(&ctx_string);
                formatted_str
            })
    }
}

impl Render for TasksPage {
    fn render(&self) -> String {
        let mut ctx = get_template_file("tasks_page").unwrap();
        ctx = ctx.replace("<%= tasks %>", &self.render_tasks());
        render_includes(ctx, None)
    }
}
