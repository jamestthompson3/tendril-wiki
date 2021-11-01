use markdown::parsers::format_links;

use crate::{get_template_file, parse_includes, process_included_file, Render};

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

impl Render for SearchResultsPage {
    fn render(&self) -> String {
        let mut ctx = get_template_file("search_results").unwrap();
        ctx = ctx.replace("<%= pages %>", &self.render_pages());
        self.render_includes(ctx)
    }
}
