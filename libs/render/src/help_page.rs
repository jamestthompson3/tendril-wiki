use crate::{get_template_file, render_includes, Render};
use async_trait::async_trait;

pub struct HelpPage {}

impl HelpPage {
    pub fn new() -> Self {
        HelpPage {}
    }
}

impl Default for HelpPage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Render for HelpPage {
    async fn render(&self) -> String {
        let ctx = get_template_file("help").await.unwrap();
        let nav = get_template_file("nav").await.unwrap();
        render_includes(ctx, None).await.replace("<%= nav %>", &nav)
    }
}
