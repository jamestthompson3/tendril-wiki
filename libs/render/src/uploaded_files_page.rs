use crate::{get_template_file, render_includes, Render};
use async_trait::async_trait;
use std::fmt::Write as _;

pub struct UploadedFilesPage {
    pub entries: Vec<String>,
}

impl UploadedFilesPage {
    pub fn new(entries: Vec<String>) -> Self {
        Self { entries }
    }
    fn render_entries(&self) -> String {
        let mut entry_list = String::new();
        for entry in &self.entries {
            write!(entry_list, "<a href=\"/files/{}\">{}</a>", entry, entry).unwrap();
        }
        entry_list
    }
}

#[async_trait]
impl Render for UploadedFilesPage {
    async fn render(&self) -> String {
        let mut ctx = get_template_file("file_list").await.unwrap();
        ctx = ctx.replace("<%= entries %>", &self.render_entries());
        render_includes(ctx, None).await
    }
}
