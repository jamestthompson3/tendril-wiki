use std::collections::HashMap;
use std::fmt::Write as _;
use std::{collections::BTreeMap, env, io, sync::Arc};

#[macro_use]
extern crate lazy_static;

use chrono::{DateTime, FixedOffset};
#[cfg(not(debug_assertions))]
use directories::ProjectDirs;

use async_trait::async_trait;
use futures::{stream, StreamExt};
use persistance::fs::read_note_cache;
use tokio::{fs, sync::Mutex};
use wikitext::parsers::{format_links, TemplattedPage};

pub mod bookmark_page;
pub mod error_page;
pub mod file_upload_page;
pub mod help_page;
pub mod index_page;
pub mod link_page;
pub mod login_page;
pub mod new_page;
pub mod opensearch_page;
pub mod search_results_page;
pub mod styles_page;
pub mod tasks_page;
pub mod uploaded_files_page;
pub mod wiki_page;

pub enum CompileState {
    Static,
    Dynamic,
}

pub type GlobalBacklinks = Arc<Mutex<BTreeMap<String, Vec<String>>>>;

const FORBIDDEN_TAGS: [&str; 5] = ["noscript", "script", "object", "embed", "link"];

#[async_trait]
pub trait Render {
    async fn render(&self) -> String;
}

lazy_static! {
    pub static ref STATIC_BUILD: String = {
        match env::var("TENDRIL_COMPILE_STATIC") {
            Ok(val) => val,
            _ => String::with_capacity(0),
        }
    };
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

pub fn sanitize_html(html: &str) -> String {
    let mut sanitized = String::from(html);
    for tag in FORBIDDEN_TAGS {
        if sanitized.contains(tag) {
            sanitized = sanitized
                .replace(&format!("<{}>", tag), &format!("&lt;{}&gt;", tag))
                .replace(&format!("</{}>", tag), &format!("&lt;/{}&gt;", tag))
                .replace(&format!("{}>", tag), &format!("{}&gt;", tag))
                .replace(&format!("<{}", tag), &format!("&lt;{}", tag))
                .replace(&format!("</{}", tag), &format!("&lt;/{}", tag));
        }
    }
    sanitized
}

async fn process_included_file(file: String, page: Option<&TemplattedPage>) -> String {
    let state = if STATIC_BUILD.as_str() == "true" {
        CompileState::Static
    } else {
        CompileState::Dynamic
    };
    match file.as_ref() {
        "search" => match state {
            CompileState::Dynamic => get_template_file("search").await.unwrap(),
            CompileState::Static => String::with_capacity(0),
        },
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
    for (key, value) in metadata.iter() {
        write!(metadata_html, "<dt><strong>{}:</strong></dt>", key).unwrap();
        // TODO: Add "created" date here as well
        // TODO: Modify dates to be compliant with DT parsing
        if key == "modified" || key == "created" {
            if let Ok(val) = value.parse::<DateTime<FixedOffset>>() {
                let val = val.format("%Y-%m-%d %H:%M").to_string();
                write!(metadata_html, "\n<dd>{}</dd>", val).unwrap();
            } else {
                write!(metadata_html, "\n<dd>{}</dd>", value).unwrap();
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
                    write!(metadata_html, "\n<dd>{}</dd>", val).unwrap();
                }
                _ => {
                    let val = format!("<a href=\"{}\">{}</a>", value, value);
                    write!(metadata_html, "\n<dd>{}</dd>", val).unwrap();
                }
            }
        } else {
            write!(metadata_html, "\n<dd>{}</dd>", &value).unwrap();
        }
    }
    metadata_html
}

pub async fn render_mru() -> String {
    let recent = read_note_cache().await;
    let mut html = String::new();
    for line in recent.lines() {
        write!(html, "<li><a href=\"{}\">{}</a></li>", line, line).unwrap();
    }
    html
}

#[cfg(debug_assertions)]
fn get_template_location(requested_file: &str) -> String {
    if requested_file.contains('.') {
        return format!("templates/{}", requested_file);
    }
    format!("templates/{}.html", requested_file)
}

pub fn render_page_backlinks(links: &[String]) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_html() {
        for tag in FORBIDDEN_TAGS {
            let test_string = format!("<{}>asdf</{}>", tag, tag);
            let result = sanitize_html(&test_string);
            assert_ne!(test_string, result);
            assert!(result.find('>').is_none());
            assert!(result.find('<').is_none());
        }
        // broken html
        for tag in FORBIDDEN_TAGS {
            let test_string = format!("<{}asdf</{}>", tag, tag);
            let result = sanitize_html(&test_string);
            assert_ne!(test_string, result);
            assert!(result.find('>').is_none());
            assert!(result.find('<').is_none());
        }
        for tag in FORBIDDEN_TAGS {
            let test_string = format!("{}>asdf</{}>", tag, tag);
            let result = sanitize_html(&test_string);
            assert_ne!(test_string, result);
            assert!(result.find('>').is_none());
            assert!(result.find('<').is_none());
        }
    }
}
