use async_trait::async_trait;
use chrono::{DateTime, FixedOffset};
use markdown::parsers::{format_links, TemplattedPage};

use crate::{get_template_file, render_includes, Render};

pub struct WikiPage<'a> {
    page: &'a TemplattedPage,
    links: Option<&'a Vec<String>>,
}

impl<'a> WikiPage<'a> {
    pub fn new(page: &'a TemplattedPage, links: Option<&'a Vec<String>>) -> Self {
        Self { page, links }
    }

    fn render_page_backlinks(&self, links: &[String]) -> String {
        if !links.is_empty() {
            let backlinks_string = links
                .iter()
                .map(|l| format!("<a href=\"{}\">{}</a>", format_links(l), l))
                .collect::<Vec<String>>()
                .join("\n");
            format!(
                r#"
<section class="backlinks-container">
  <hr />
  <h3>Mentioned in:</h3>
  <div class="backlinks">{}</div>
</section>
"#,
                backlinks_string
            )
        } else {
            String::with_capacity(0)
        }
    }

    fn render_page_metadata(&self) -> String {
        let mut metadata_html = String::new();
        for (key, value) in self.page.metadata.iter() {
            metadata_html.push_str(&format!("<dt><strong>{}:</strong></dt>", key));
            // TODO: Add "created" date here as well
            // TODO: Modify dates to be compliant with DT parsing
            if key == "modified" || key == "created" {
                if let Ok(val) = value.parse::<DateTime<FixedOffset>>() {
                    let val = val.format("%Y-%m-%d %H:%M").to_string();
                    metadata_html.push_str(&format!("\n<dd>{}</dd>", val));
                } else {
                    metadata_html.push_str(&format!("\n<dd>{}</dd>", value));
                }
                continue;
            }
            if value.starts_with("http") {
                match key.as_str() {
                    "cover" => {
                        let val = format!(
                            "<img src=\"{}\" style=\"max-height: 200px; max-width: 200px;\">",
                            value
                        );
                        metadata_html.push_str(&format!("\n<dd>{}</dd>", val));
                    }
                    _ => {
                        let val = format!("<a href=\"{}\">{}</a>", value, value);
                        metadata_html.push_str(&format!("\n<dd>{}</dd>", val));
                    }
                }
            } else {
                metadata_html.push_str(&format!("\n<dd>{}</dd>", &value));
            }
        }
        metadata_html
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
        ctx = ctx
            .replace("<%= title %>", &page.title)
            .replace("<%= body %>", &page.body)
            .replace("<%= tags %>", &tag_string)
            .replace("<%= links %>", &self.render_page_backlinks(&backlinks))
            .replace("<%= metadata %>", &self.render_page_metadata());
        render_includes(ctx, Some(self.page)).await
    }
}
