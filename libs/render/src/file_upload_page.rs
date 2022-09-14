use crate::{get_template_file, render_includes, render_sidebar, Render};
use async_trait::async_trait;

pub struct FileUploader {}

impl Default for FileUploader {
    fn default() -> Self {
        Self::new()
    }
}

impl FileUploader {
    pub fn new() -> Self {
        FileUploader {}
    }
}

#[async_trait]
impl Render for FileUploader {
    async fn render(&self) -> String {
        let mut ctx = get_template_file("file_upload").await.unwrap();
        let nav = get_template_file("nav").await.unwrap();
        ctx = ctx.replace("<%= sidebar %>", &render_sidebar().await);
        render_includes(ctx, None).await.replace("<%= nav %>", &nav)
    }
}
