use async_trait::async_trait;
use std::fmt::Write as _;
use wikitext::parsers::format_links;

use crate::{get_template_file, render_includes, Render};

type SearchResult = Vec<(String, String)>;

pub struct SearchResultsPage {
    pub pages: SearchResult,
}

impl SearchResultsPage {
    pub fn new(pages: SearchResult) -> Self {
        SearchResultsPage { pages }
    }
    async fn render_pages(&self) -> String {
        if self.pages.is_empty() {
            return String::from("<h3>No search results.</h3>");
        }
        let mut page_list = String::new();
        for (page, matched_text) in self.pages.iter() {
            write!(
                page_list,
                "<li><div class=\"result\"><h2><a href=\"{}\">{}</a></h2>",
                format_links(page),
                page,
            )
            .unwrap();
            page_list.push_str(matched_text);
            page_list.push_str("</div></li>");
        }
        page_list
    }
}

#[async_trait]
impl Render for SearchResultsPage {
    async fn render(&self) -> String {
        let mut ctx = get_template_file("search_results").await.unwrap();
        ctx = ctx.replace("<%= pages %>", &self.render_pages().await);
        render_includes(ctx, None).await
    }
}
