use markdown::parsers::format_links;
use tasks::{CompileState, SearchResult};

use crate::{get_template_file, render_includes, Render};

pub struct SearchResultsContextPage {
    pub pages: Vec<SearchResult>,
}

impl SearchResultsContextPage {
    pub fn new(pages: Vec<SearchResult>) -> Self {
        SearchResultsContextPage { pages }
    }
    fn render_pages(&self) -> String {
        let mut page_list = String::new();
        for page in &self.pages {
            page_list.push_str(&format!(
                "<li><h4><a href=\"{}\">{}</a></h4><p>{}</p></li>",
                format_links(&page.location),
                page.location,
                page.matched_text
            ));
        }
        page_list
    }
}

impl Render for SearchResultsContextPage {
    fn render(&self, state: &CompileState) -> String {
        let mut ctx = get_template_file("search_results_context").unwrap();
        ctx = ctx.replace("<%= pages %>", &self.render_pages());
        render_includes(ctx, state)
    }
}
