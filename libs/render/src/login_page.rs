use crate::{get_template_file, render_includes, Render};
use async_trait::async_trait;

pub struct LoginPage {}

impl LoginPage {
    pub fn new() -> Self {
        LoginPage {}
    }
}

impl Default for LoginPage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Render for LoginPage {
    async fn render(&self) -> String {
        let ctx = get_template_file("login").await.unwrap();
        render_includes(ctx, None).await
    }
}
