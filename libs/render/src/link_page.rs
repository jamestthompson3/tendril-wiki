use async_trait::async_trait;
use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::{get_template_file, render_includes, Render};
use markdown::parsers::format_links;

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
            write!(html, "<h2>{}</h2><ul>", key).unwrap();
            let mut links = self.links.get(key).unwrap().to_owned();
            links.dedup();
            for link in links {
                write!(
                    html,
                    "<li><a href=\"{}\">{}</a></li>",
                    format_links(&link),
                    link
                )
                .unwrap();
            }
            write!(html, "</ul>").unwrap();
            link_content.push(html);
        }
        link_content.join("")
    }
}

#[async_trait]
impl Render for LinkPage {
    async fn render(&self) -> String {
        let mut ctx = get_template_file("backlinks").await.unwrap();
        ctx = ctx.replace("<%= link_content %>", &self.create_link_content());
        render_includes(ctx, None).await
    }
}
