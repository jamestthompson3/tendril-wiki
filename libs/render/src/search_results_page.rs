use async_trait::async_trait;
use std::{fmt::Write as _, time::Duration};
use wikitext::parsers::format_links;

use crate::{get_template_file, render_includes, Render};

type SearchResult = Vec<String>;

pub struct SearchResultsPage {
    pub pages: SearchResult,
    pub num_results: usize,
    pub time: Duration,
}

impl SearchResultsPage {
    pub fn new(pages: SearchResult, num_results: usize, time: Duration) -> Self {
        SearchResultsPage {
            pages,
            num_results,
            time,
        }
    }
    async fn render_pages(&self) -> String {
        if self.pages.is_empty() {
            return String::with_capacity(0);
        }
        let mut page_list = String::new();
        for page in self.pages.iter() {
            write!(
                page_list,
                "<li><div class=\"result\"><h2><a href=\"{}\">{}</a></h2><button class=\"expand\">&#9660;</button></div></li>",
                format_links(page),
                page,
            )
            .unwrap();
        }
        page_list
    }
    fn render_result_header(&self) -> String {
        if self.pages.is_empty() {
            return String::from("<h3>No search results.</h3>");
        }
        let mut result_header = String::new();
        write!(
            result_header,
            r#"<h4><strong>{}</strong> results in <strong>{:?}</strong>"#,
            self.num_results, self.time
        )
        .unwrap();
        result_header
    }
}

#[async_trait]
impl Render for SearchResultsPage {
    async fn render(&self) -> String {
        let nav = get_template_file("nav").await.unwrap();
        let mut ctx = get_template_file("search_results").await.unwrap();
        ctx = ctx
            .replace("<%= pages %>", &self.render_pages().await)
            .replace("<%= result_header %>", &self.render_result_header());
        render_includes(ctx, None).await.replace("<%= nav %>", &nav)
    }
}
