use tasks::CompileState;

use crate::{get_template_file, render_includes, Render};

pub struct LoginPage {}

impl LoginPage {
    pub fn new() -> Self {
        LoginPage {}
    }
}

impl Render for LoginPage {
    fn render(&self, state: &CompileState) -> String {
        let ctx = get_template_file("login").unwrap();
        render_includes(ctx, state, None)
    }
}