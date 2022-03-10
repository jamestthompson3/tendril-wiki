use crate::{get_template_file, render_includes, Render};

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

impl Render for LoginPage {
    fn render(&self) -> String {
        let ctx = get_template_file("login").unwrap();
        render_includes(ctx, None)
    }
}
