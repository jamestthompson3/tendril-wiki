use std::collections::BTreeMap;

use markdown::parsers::format_links;

use crate::{get_template_file, parse_includes, process_included_file, Render};

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
    fn render_includes(&self, ctx: String) -> String {
        let lines = ctx.lines().map(|line| {
            let line = line.trim();
            if line.starts_with("<%=") {
                process_included_file(parse_includes(line), None)
            } else {
                line.to_string()
            }
        });
        lines.collect::<Vec<String>>().join(" ")
    }
}

impl Render for TagIndex {
    fn render(&self) -> String {
        let mut ctx = get_template_file("tag_idx").unwrap();
        ctx = ctx.replace("<%= tag_idx %>", &self.render_tag_body());
        self.render_includes(ctx)
    }
}
