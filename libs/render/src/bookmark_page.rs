use crate::{get_template_file, render_includes, Render};
use async_trait::async_trait;

pub struct BookmarkAddPage {}

impl Default for BookmarkAddPage {
    fn default() -> Self {
        Self::new()
    }
}

impl BookmarkAddPage {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Render for BookmarkAddPage {
    async fn render(&self) -> String {
        let ctx = get_template_file("bookmark_add").await.unwrap();
        let nav = get_template_file("nav").await.unwrap();
        render_includes(ctx, None).await.replace("<%= nav %>", &nav)
    }
}
