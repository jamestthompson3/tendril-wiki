use async_trait::async_trait;
use markdown::parsers::format_links;
use tasks::SearchResult;

use crate::{get_template_file, render_includes, Render};

pub struct SearchResultsPage {
    pub pages: Vec<SearchResult>,
}

impl SearchResultsPage {
    pub fn new(pages: Vec<SearchResult>) -> Self {
        SearchResultsPage { pages }
    }
    async fn render_pages(&self) -> String {
        if self.pages.is_empty() {
            let mut ctx = String::from("<h3>No search results.</h3>");
            ctx.push_str(
                &render_includes(r#"<%= include "search_form" %>"#.to_string(), None).await,
            );
            return ctx;
        }
        let mut page_list = String::new();
        for result in self.pages.iter() {
            for (page, matched_text) in result.iter() {
                page_list.push_str(&format!(
                    "<li><a href=\"{}\">{}</a>",
                    format_links(page),
                    page,
                ));
                for text in matched_text {
                    page_list.push_str(&format!("<p>{}</p>", text));
                }
                page_list.push_str("</li>");
            }
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
