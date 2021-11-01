use crate::{get_template_file, Render};

pub struct LoginPage {}

impl LoginPage {
    pub fn new() -> Self {
        LoginPage {}
    }
}

impl Render for LoginPage {
    fn render(&self) -> String {
        // TODO: parse_includes
        get_template_file("login").unwrap()
    }
}
