use async_trait::async_trait;

use wikitext::parsers::TemplattedPage;

use crate::{
    get_template_file, render_includes, render_page_backlinks, render_page_metadata,
    PageRenderLinks, Render,
};

pub struct StaticSitePage<'a> {
    page: &'a TemplattedPage,
    links: PageRenderLinks<'a>,
}

impl<'a> StaticSitePage<'a> {
    pub fn new(page: &'a TemplattedPage, links: PageRenderLinks<'a>) -> Self {
        Self { page, links }
    }
}

#[async_trait]
impl<'a> Render for StaticSitePage<'a> {
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
        let mut ctx = get_template_file("static_site").await.unwrap();
        let content = get_template_file("content").await.unwrap();
        ctx = ctx
            .replace("<%= content %>", &content)
            .replace("<%= body %>", &page.body)
            .replace("<%= tags %>", &tag_string)
            .replace("<%= links %>", &render_page_backlinks(backlinks))
            .replace("<%= title %>", &page.title)
            .replace(
                "<%= metadata %>",
                &render_page_metadata(page.metadata.clone()),
            );
        render_includes(ctx, Some(page)).await
    }
}
