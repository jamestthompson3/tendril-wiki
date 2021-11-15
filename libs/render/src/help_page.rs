use tasks::CompileState;

use crate::{get_template_file, render_includes, Render};

pub struct HelpPage {}

impl HelpPage {
    pub fn new() -> Self {
        HelpPage {}
    }
}

impl Render for HelpPage {
    fn render(&self, state: &CompileState) -> String {
        let ctx = get_template_file("help").unwrap();
        render_includes(ctx, state, None)
    }
}
