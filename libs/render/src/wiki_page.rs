use async_trait::async_trait;

use wikitext::parsers::TemplattedPage;

use crate::{
    get_template_file, render_includes, render_page_backlinks, render_page_metadata,
    render_sidebar, PageRenderLinks, Render,
};

pub struct WikiPage<'a> {
    page: &'a TemplattedPage,
    links: PageRenderLinks<'a>,
}

impl<'a> WikiPage<'a> {
    pub fn new(page: &'a TemplattedPage, links: PageRenderLinks<'a>) -> Self {
        Self { page, links }
    }

    fn render_body(&self) -> String {
        self.page
            .body
            .split('\n')
            .map(|line| line.to_owned())
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
        let content = get_template_file("content").await.unwrap();
        let nav = get_template_file("nav").await.unwrap();
        ctx = ctx
            .replace("<%= sidebar %>", &render_sidebar().await)
            .replace("<%= content %>", &content)
            .replace("<%= tags %>", &tag_string)
            .replace("<%= links %>", &render_page_backlinks(backlinks))
            .replace("<%= body %>", &self.render_body())
            .replace(
                "<%= metadata %>",
                &render_page_metadata(page.metadata.clone()),
            );
        render_includes(ctx, Some(page))
            .await
            .replace("<%= nav %>", &nav)
            .replace("<%= title %>", &page.title)
    }
}
