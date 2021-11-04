use std::collections::BTreeMap;

use markdown::parsers::format_links;
use tasks::CompileState;

use crate::{get_template_file, render_includes, Render};

pub struct TagIndex {
    pub tags: BTreeMap<String, Vec<String>>,
}

impl TagIndex {
    pub fn new(tags: BTreeMap<String, Vec<String>>) -> Self {
        TagIndex { tags }
    }
    fn render_tag_body(&self) -> String {
        let mut tag_content: Vec<String> = Vec::with_capacity(self.tags.len());
        for key in self.tags.keys() {
            let mut html = String::new();
            html.push_str(&format!(
                "<h2><a href=\"/tags/{}\" class=\"block\">{}</a></h2><ul>",
                key, key
            ));
            let mut tags = self.tags.get(key).unwrap().to_owned();
            tags.dedup();
            for tag in tags {
                html.push_str(&format!(
                    "<li><a href=\"{}\">{}</a></li>",
                    format_links(&tag),
                    tag
                ));
            }
            html.push_str("</ul>");
            tag_content.push(html);
        }
        tag_content.join("")
    }
}

impl Render for TagIndex {
    fn render(&self, state: &CompileState) -> String {
        let mut ctx = get_template_file("tag_idx").unwrap();
        ctx = ctx.replace("<%= tag_idx %>", &self.render_tag_body());
        render_includes(ctx, state)
    }
}
