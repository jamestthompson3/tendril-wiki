use crate::{get_template_file, render_includes, Render};
use async_trait::async_trait;

pub struct OpenSearchPage {
    pub user: String,
    pub host: String,
}

impl OpenSearchPage {
    pub fn new(user: String, host: String) -> Self {
        OpenSearchPage { user, host }
    }
}

#[async_trait]
impl Render for OpenSearchPage {
    async fn render(&self) -> String {
        let mut ctx = get_template_file("opensearchdescription.xml").await.unwrap();
        ctx = ctx.replace("<%= user %>", &self.user);
        ctx = ctx.replace("<%= host %>", &self.host);
        render_includes(ctx, None).await
    }
}
