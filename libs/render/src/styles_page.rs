use crate::{get_template_file, render_includes, Render, render_sidebar};
use async_trait::async_trait;

pub struct StylesPage {
    pub body: String,
}

impl StylesPage {
    pub fn new(body: String) -> Self {
        StylesPage { body }
    }
}

#[async_trait]
impl Render for StylesPage {
    async fn render(&self) -> String {
        let mut ctx = get_template_file("user_style").await.unwrap();
        let nav = get_template_file("nav").await.unwrap();
        ctx = ctx
            .replace("<%= body %>", &self.body)
            .replace("<%= sidebar %>", &render_sidebar().await);
        render_includes(ctx, None).await.replace("<%= nav %>", &nav)
    }
}
