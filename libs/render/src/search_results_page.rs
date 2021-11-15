use markdown::parsers::format_links;
use tasks::CompileState;

use crate::{get_template_file, render_includes, Render};

pub struct SearchResultsPage {
    pub pages: Vec<String>,
}

impl SearchResultsPage {
    pub fn new(pages: Vec<String>) -> Self {
        SearchResultsPage { pages }
    }
    fn render_pages(&self) -> String {
        let mut page_list = String::new();
        for page in &self.pages {
            page_list.push_str(&format!(
                "<li><a href=\"{}\">{}</a></li>",
                format_links(page),
                page
            ));
        }
        page_list
    }
}

impl Render for SearchResultsPage {
    fn render(&self, state: &CompileState) -> String {
        let mut ctx = get_template_file("search_results").unwrap();
        ctx = ctx.replace("<%= pages %>", &self.render_pages());
        render_includes(ctx, state, None)
    }
}
