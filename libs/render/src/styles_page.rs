use crate::{get_template_file, render_includes, Render};

pub struct StylesPage {
    pub body: String,
}

impl StylesPage {
    pub fn new(body: String) -> Self {
        StylesPage { body }
    }
}

impl Render for StylesPage {
    fn render(&self) -> String {
        let mut ctx = get_template_file("user_style").unwrap();
        ctx = ctx.replace("<%= body %>", &self.body);
        render_includes(ctx, None)
    }
}
