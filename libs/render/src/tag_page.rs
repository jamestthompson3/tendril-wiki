use tasks::CompileState;

use crate::{get_template_file, render_includes, Render};

pub struct TagPage {
    pub title: String,
    pub tags: Vec<String>,
}

impl TagPage {
    pub fn new(title: String, tags: Vec<String>) -> Self {
        TagPage { title, tags }
    }
}

impl Render for TagPage {
    fn render(&self, state: &CompileState) -> String {
        let mut ctx = get_template_file("tags").unwrap();
        let tag_string = self
            .tags
            .iter()
            .map(|t| format!("<li><a href=\"/{}\">{}</a></li>", t, t))
            .collect::<Vec<String>>()
            .join("\n");
        ctx = ctx
            .replace("<%= title %>", &self.title)
            .replace("<%= tags %>", &tag_string);
        render_includes(ctx, state, None)
    }
}
