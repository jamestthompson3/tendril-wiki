use tasks::CompileState;

use crate::{get_template_file, render_includes, Render};

pub struct SearchPage {}

impl SearchPage {
    pub fn new() -> Self {
        SearchPage {}
    }
}

impl Render for SearchPage {
    fn render(&self, state: &CompileState) -> String {
        let ctx = get_template_file("search").unwrap();
        render_includes(ctx, state)
    }
}
