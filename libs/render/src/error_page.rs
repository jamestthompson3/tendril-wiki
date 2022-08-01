use crate::{get_template_file, render_includes, Render};
use async_trait::async_trait;

pub struct ErrorPage {
    pub msg: String
}


impl ErrorPage {
    pub fn new(msg: String) -> Self {
        Self { msg }
    }
}

#[async_trait]
impl Render for ErrorPage {
    async fn render(&self) -> String {
        let mut ctx = get_template_file("error_page").await.unwrap();
        ctx = ctx.replace("<%= msg %>", &self.msg);
        render_includes(ctx, None).await
    }
}
