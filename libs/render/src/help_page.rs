use crate::{get_template_file, render_includes, Render};

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

impl Render for HelpPage {
    fn render(&self) -> String {
        let ctx = get_template_file("help").unwrap();
        render_includes(ctx, None)
    }
}
