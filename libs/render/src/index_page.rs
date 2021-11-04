use tasks::CompileState;

use crate::{get_template_file, render_includes, Render};

pub struct IndexPage {
    pub user: String,
}

impl IndexPage {
    pub fn new(user: String) -> Self {
        IndexPage { user }
    }
}

impl Render for IndexPage {
    fn render(&self, state: &CompileState) -> String {
        let mut ctx = get_template_file("index").unwrap();
        ctx = ctx.replace("<%= user %>", &self.user);
        render_includes(ctx, state)
    }
}
