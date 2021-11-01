use markdown::parsers::{format_links, TemplattedPage};

use crate::{get_template_file, parse_includes, process_included_file, Render};

pub struct WikiPage<'a> {
    page: &'a TemplattedPage,
    links: Option<&'a Vec<String>>,
    render_static: bool,
}

impl<'a> WikiPage<'a> {
    pub fn new(
        page: &'a TemplattedPage,
        links: Option<&'a Vec<String>>,
        render_static: bool,
    ) -> Self {
        Self {
            page,
            links,
            render_static,
        }
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
            // if key == "modified" {
            //     val = value.parse::<DateTime<FixedOffset>>().unwrap().format("%Y-%m-%d %H:%M").to_string();
            //   }
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

impl<'a> Render for WikiPage<'a> {
    fn render(&self) -> String {
        let page = self.page;
        let mut backlinks = match self.links {
            Some(links) => links.to_owned(),
            None => Vec::new(),
        };
        backlinks.dedup();
        let tag_string = page
            .tags
            .iter()
            .map(|t| format!("<li><a href=\"/tags/{}\">#{}</a></li>", t, t))
            .collect::<Vec<String>>()
            .join("\n");
        let mut ctx = get_template_file("main").unwrap();
        ctx = ctx
            .replace("<%= title %>", &page.title)
            .replace("<%= body %>", &page.body)
            .replace("<%= tags %>", &tag_string)
            .replace("<%= links %>", &self.render_page_backlinks(&backlinks))
            .replace("<%= metadata %>", &self.render_page_metadata());
        let parsed = ctx.split('\n');
        parsed
            .map(|line| {
                if line.trim().starts_with("<%= include") {
                    let included_file = parse_includes(line.trim());
                    match included_file.as_ref() {
                        "nav" | "edit" => {
                            if self.render_static {
                                return String::with_capacity(0);
                            }
                            process_included_file(included_file, Some(page))
                        }

                        _ => get_template_file(&included_file).unwrap(),
                    }
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<String>>()
            .join("\n")
    }
}
