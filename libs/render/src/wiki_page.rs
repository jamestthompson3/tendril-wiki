use async_trait::async_trait;

use wikitext::parsers::TemplattedPage;

use crate::{
    get_template_file, render_includes, render_mru, render_page_backlinks, render_page_metadata,
    Render,
};

pub struct WikiPage<'a> {
    page: &'a TemplattedPage,
    links: Option<&'a Vec<String>>,
}

impl<'a> WikiPage<'a> {
    pub fn new(page: &'a TemplattedPage, links: Option<&'a Vec<String>>) -> Self {
        Self { page, links }
    }

    fn render_body(&self) -> String {
        self.page
            .body
            .split('\n')
            .filter_map(|line| {
                if line.is_empty() {
                    None
                } else {
                    Some(line.to_owned())
                }
            })
            .collect::<Vec<String>>()
            .join("\n")
    }
}

#[async_trait]
impl<'a> Render for WikiPage<'a> {
    async fn render(&self) -> String {
        let page = self.page;
        let mut backlinks = match self.links {
            Some(links) => links.to_owned(),
            None => Vec::new(),
        };
        backlinks.dedup();
        backlinks.sort_unstable();
        let tag_string = page
            .tags
            .iter()
            .map(|t| format!("<li><a href=\"{}\">#{}</a></li>", t, t))
            .collect::<Vec<String>>()
            .join("\n");
        let mut ctx = get_template_file("main").await.unwrap();
        let sidebar = get_template_file("sidebar").await.unwrap();
        let content = get_template_file("content").await.unwrap();
        ctx = ctx
            .replace("<%= sidebar %>", &sidebar)
            .replace("<%= content %>", &content)
            .replace("<%= tags %>", &tag_string)
            .replace("<%= links %>", &render_page_backlinks(&backlinks))
            .replace("<%= mru %>", &render_mru().await)
            .replace("<%= body %>", &self.render_body())
            .replace("<%= title %>", &page.title)
            .replace(
                "<%= metadata %>",
                &render_page_metadata(self.page.metadata.clone()),
            );
        render_includes(ctx, Some(self.page)).await
    }
}
