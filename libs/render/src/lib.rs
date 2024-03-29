use std::collections::HashMap;
use std::fmt::Write as _;
use std::io;

use chrono::{DateTime, FixedOffset};
#[cfg(not(debug_assertions))]
use directories::ProjectDirs;

use async_trait::async_trait;
use futures::{stream, StreamExt};
use tokio::fs;
use wikitext::parsers::{format_links, TemplattedPage};

pub mod all_pages;
pub mod bookmark_page;
pub mod error_page;
pub mod file_upload_page;
pub mod help_page;
pub mod index_page;
pub mod injected_html;
pub mod login_page;
pub mod new_page;
pub mod opensearch_page;
pub mod search_results_page;
pub mod static_site_page;
pub mod styles_page;
pub mod tasks_page;
pub mod uploaded_files_page;
pub mod wiki_page;

pub enum CompileState {
    Static,
    Dynamic,
}

pub type PageRenderLinks<'a> = Option<&'a Vec<String>>;

#[async_trait]
pub trait Render {
    async fn render(&self) -> String;
}

pub fn parse_includes(include_str: &str) -> String {
    let included_file = include_str
        .trim()
        .strip_prefix("<%= include \"")
        .unwrap()
        .strip_suffix("\" %>")
        .unwrap();
    included_file.to_string()
}

async fn process_included_file(file: String, page: Option<&TemplattedPage>) -> String {
    match file.as_ref() {
        "search" => get_template_file("search").await.unwrap(),
        "styles" => get_template_file("styles").await.unwrap(),
        "meta" => {
            let templatefile = get_template_file("meta").await.unwrap();
            let page = page.unwrap();
            let icon_path = match &page.metadata.get("icon") {
                Some(icon) => format!("/files/{}", icon),
                None => String::from("static/favicon.ico"),
            };
            templatefile
                .replace("<%= title %>", &page.title)
                .replace("<%= desc %>", &page.desc)
                .replace("<%= icon %>", &icon_path)
        }
        "footer" => get_template_file("footer").await.unwrap(),
        _ => String::with_capacity(0),
    }
}

pub async fn render_includes(ctx: String, page: Option<&TemplattedPage>) -> String {
    let stream = stream::iter(ctx.lines());
    let file_lines = stream.then(|line| async {
        if line.contains("<%= include") {
            process_included_file(parse_includes(line), page).await
        } else {
            line.to_string()
        }
    });
    let collected = file_lines.collect::<Vec<String>>().await;
    collected.join("\n")
}

pub async fn get_template_file(requested_file: &str) -> Result<String, io::Error> {
    let file_path = get_template_location(requested_file);
    if let Ok(filestring) = fs::read_to_string(&file_path).await {
        Ok(filestring)
    } else {
        eprintln!("Could not find {}", requested_file);
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Could not find {}", requested_file),
        ))
    }
}

pub fn render_page_metadata(metadata: HashMap<String, String>) -> String {
    let mut metadata_html = String::new();
    if metadata.is_empty() {
        return metadata_html;
    }
    for (key, value) in metadata.iter() {
        write!(metadata_html, "<dt>{}</dt>", key).unwrap();
        // TODO: Add "created" date here as well
        // TODO: Modify dates to be compliant with DT parsing
        match key.as_str() {
            "modified" | "created" => {
                if let Ok(val) = value.parse::<DateTime<FixedOffset>>() {
                    let val = val.format("%Y-%m-%d %H:%M").to_string();
                    write!(metadata_html, "<dd>{}</dd>", val).unwrap();
                } else {
                    write!(metadata_html, "<dd>{}</dd>", value).unwrap();
                }
            }
            "cover" => {
                if value.starts_with("http") || value.starts_with("file://") {
                    let val = format!("<img src=\"{}\">", value);
                    write!(metadata_html, "<dd>{}</dd>", val).unwrap();
                }
            }
            "isbn" => {
                write!(
                    metadata_html,
                    "<dd>{}<br><img src=\"https://covers.openlibrary.org/b/isbn/{}-M.jpg\"></dd>",
                    &value, value
                )
                .unwrap();
            }
            _ => {
                if value.starts_with("http") || value.starts_with("file://") {
                    let val = format!("<a href=\"{}\">{}</a>", value, value);
                    write!(metadata_html, "<dd>{}</dd>", val).unwrap();
                } else {
                    write!(metadata_html, "<dd>{}</dd>", &value).unwrap();
                }
            }
        }
    }
    metadata_html
}


#[cfg(debug_assertions)]
fn get_template_location(requested_file: &str) -> String {
    if requested_file.contains('.') {
        return format!("templates/{}", requested_file);
    }
    format!("templates/{}.html", requested_file)
}

pub fn render_page_backlinks(links: Vec<String>) -> String {
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

#[cfg(not(debug_assertions))]
fn get_template_location(requested_file: &str) -> String {
    let project_dir = ProjectDirs::from("", "", "tendril").unwrap();
    let mut data_dir = project_dir.data_dir().to_owned();

    if requested_file.contains('.') {
        data_dir.push(format!("templates/{}", requested_file));
    } else {
        data_dir.push(format!("templates/{}.html", requested_file));
    }
    data_dir.to_string_lossy().into()
}
