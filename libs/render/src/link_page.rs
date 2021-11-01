use std::collections::BTreeMap;

use markdown::parsers::format_links;

use crate::{get_template_file, parse_includes, process_included_file, Render};

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

impl Render for LinkPage {
    fn render(&self) -> String {
        let mut ctx = get_template_file("backlinks").unwrap();
        ctx = ctx.replace("<%= link_content %>", &self.create_link_content());
        self.render_includes(ctx)
    }
}
