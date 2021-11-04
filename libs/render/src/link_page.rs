use std::collections::BTreeMap;

use markdown::parsers::format_links;
use tasks::CompileState;

use crate::{get_template_file, render_includes, Render};

pub struct LinkPage {
    pub links: BTreeMap<String, Vec<String>>,
}

impl LinkPage {
    pub fn new(links: BTreeMap<String, Vec<String>>) -> Self {
        LinkPage { links }
    }
    fn create_link_content(&self) -> String {
        let mut link_content: Vec<String> = Vec::with_capacity(self.links.len());
        for key in self.links.keys() {
            let mut html = String::new();
            html.push_str(&format!("<h2>{}</h2><ul>", key));
            let mut links = self.links.get(key).unwrap().to_owned();
            links.dedup();
            for link in links {
                html.push_str(&format!(
                    "<li><a href=\"{}\">{}</a></li>",
                    format_links(&link),
                    link
                ));
            }
            html.push_str("</ul>");
            link_content.push(html);
        }
        link_content.join("")
    }
}

impl Render for LinkPage {
    fn render(&self, state: &CompileState) -> String {
        let mut ctx = get_template_file("backlinks").unwrap();
        ctx = ctx.replace("<%= link_content %>", &self.create_link_content());
        render_includes(ctx, state)
    }
}
