use markdown::parsers::format_links;
use tasks::SearchResult;

use crate::{get_template_file, parse_includes, process_included_file, Render};

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

    fn render_includes(&self, ctx: String) -> String {
        let lines = ctx.lines().map(|line| {
            let line = line.trim();
            if line.starts_with("<%=") {
                process_included_file(parse_includes(line), None)
            } else {
                line.to_string()
            }
        });
        lines.collect::<Vec<String>>().join(" ")
    }
}

impl Render for SearchResultsContextPage {
    fn render(&self) -> String {
        let mut ctx = get_template_file("search_results_context").unwrap();
        ctx = ctx.replace("<%= pages %>", &self.render_pages());
        self.render_includes(ctx)
    }
}
