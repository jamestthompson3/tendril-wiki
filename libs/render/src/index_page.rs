use crate::{get_template_file, render_includes, Render};
use async_trait::async_trait;
use directories::ProjectDirs;

pub struct IndexPage {
    pub user: String,
    pub host: String,
}

impl IndexPage {
    pub fn new(user: String, host: String) -> Self {
        IndexPage { user, host }
    }
    fn render_mru(&self, recent: String) -> String {
        recent
            .lines()
            .map(|l| format!("<li><a href=\"{}\">{}</a></li>", l, l))
            .collect::<Vec<String>>()
            .join("")
    }
}

#[async_trait]
impl Render for IndexPage {
    async fn render(&self) -> String {
        use chrono::Local;
        let now = Local::now();
        let project_dir = ProjectDirs::from("", "", "tendril").unwrap();
        let mut data_dir = project_dir.data_dir().to_owned();
        data_dir.push("note_cache");
        let recent = tokio::fs::read_to_string(&data_dir).await;
        let mut ctx = get_template_file("index").await.unwrap();
        ctx = ctx.replace("<%= user %>", &self.user);
        ctx = ctx.replace("<%= today %>", &now.format("%Y-%m-%d").to_string());
        ctx = ctx.replace("<%= host %>", &self.host);
        ctx = ctx.replace(
            "<%= mru %>",
            &self.render_mru(recent.expect("Could not read cache file")),
        );
        render_includes(ctx, None).await
    }
}
