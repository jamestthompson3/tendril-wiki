use std::fs;

use crate::{get_template_file, render_includes, Render};
use directories::ProjectDirs;

pub struct IndexPage {
    pub user: String,
}

impl IndexPage {
    pub fn new(user: String) -> Self {
        IndexPage { user }
    }
    fn render_mru(&self, recent: String) -> String {
        recent
            .lines()
            .map(|l| format!("<li><a href=\"{}\">{}</a></li>", l, l))
            .collect::<Vec<String>>()
            .join("")
    }
}

impl Render for IndexPage {
    fn render(&self) -> String {
        use chrono::Local;
        let now = Local::now();
        let project_dir = ProjectDirs::from("", "", "tendril").unwrap();
        let mut data_dir = project_dir.data_dir().to_owned();
        data_dir.push("note_cache");
        let recent = fs::read_to_string(&data_dir);
        let mut ctx = get_template_file("index").unwrap();
        ctx = ctx.replace("<%= user %>", &self.user);
        ctx = ctx.replace("<%= today %>", &now.format("%Y-%m-%d").to_string());
        ctx = ctx.replace(
            "<%= mru %>",
            &self.render_mru(recent.expect("Could not read cache file")),
        );
        render_includes(ctx, None)
    }
}
