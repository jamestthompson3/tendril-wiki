use async_trait::async_trait;
use markdown::parsers::format_links;

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
            let mut ctx = String::from("<h3>No search results.</h3>");
            ctx.push_str(
                &render_includes(r#"<%= include "search_form" %>"#.to_string(), None).await,
            );
            return ctx;
        }
        let mut page_list = String::new();
        for (page, matched_text) in self.pages.iter() {
            page_list.push_str(&format!(
                "<li><div class=\"result\"><h2><a href=\"{}\">{}</a></h2>",
                format_links(page),
                page,
            ));
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
