use std::{fs, io};

use link_page::LinkPage;
use markdown::parsers::{GlobalBacklinks, ParsedPages, TagMapping, TemplattedPage};
use tag_index_page::TagIndex;
use tasks::CompileState;

use crate::{tag_page::TagPage, wiki_page::WikiPage};

pub mod file_upload_page;
pub mod help_page;
pub mod index_page;
pub mod link_page;
pub mod login_page;
pub mod new_page;
pub mod search_page;
pub mod search_results_context_page;
pub mod search_results_page;
pub mod styles_page;
pub mod tag_index_page;
pub mod tag_page;
pub mod uploaded_files_page;
pub mod wiki_page;

pub trait Render {
    fn render(&self, state: &CompileState) -> String;
}

pub fn parse_includes(include_str: &str) -> String {
    let included_file = include_str
        .strip_prefix("<%= include \"")
        .unwrap()
        .strip_suffix("\" %>")
        .unwrap();
    included_file.to_string()
}

#[inline]
pub fn write_tag_pages(map: TagMapping, pages: &ParsedPages, state: CompileState) {
    let tag_map = map.lock().unwrap();
    for key in tag_map.keys() {
        let title = key.to_string();
        let tags = tag_map.get(key).unwrap().to_owned();
        let pages = pages.lock().unwrap();
        let page = pages.iter().find(|pg| pg.title == title);
        if let Some(template) = page {
            let output = WikiPage::new(template, Some(&tags)).render(&state);
            fs::create_dir(format!("public/tags/{}", title)).unwrap();
            fs::write(format!("public/tags/{}/index.html", title), output).unwrap();
        } else {
            let ctx = TagPage {
                title: title.clone(),
                tags,
            };
            fs::create_dir(format!("public/tags/{}", title)).unwrap();
            fs::write(
                format!("public/tags/{}/index.html", title),
                ctx.render(&state),
            )
            .unwrap();
        }
    }
}

#[inline]
fn process_included_file(
    file: String,
    page: Option<&TemplattedPage>,
    state: &CompileState,
) -> String {
    match file.as_ref() {
        "nav" => match state {
            CompileState::Dynamic => get_template_file("nav").unwrap(),
            CompileState::Static => String::with_capacity(0),
        },
        "edit" => match state {
            CompileState::Dynamic => {
                let page = page.unwrap();
                let templatefile = get_template_file("edit").unwrap();
                let metadata_string = page
                    .metadata
                    .iter()
                    .map(|(k, v)| format!("{}:{}", k, v))
                    .collect::<Vec<String>>()
                    .join("\n");
                templatefile
                    .replace("<%= title %>", &page.title)
                    .replace("<%= tags %>", &page.tags.join(","))
                    .replace("<%= raw_md %>", &page.raw_md)
                    .replace("<%= metadata %>", &metadata_string)
            }
            CompileState::Static => String::with_capacity(0),
        },
        "styles" => get_template_file("styles").unwrap(),
        "footer" => get_template_file("footer").unwrap(),
        _ => String::with_capacity(0),
    }
}

pub fn render_includes(ctx: String, state: &CompileState) -> String {
    let lines = ctx.lines().map(|line| {
        let line = line.trim();
        if line.starts_with("<%=") {
            process_included_file(parse_includes(line), None, state)
        } else {
            line.to_string()
        }
    });
    lines.collect::<Vec<String>>().join(" ")
}

#[inline]
pub fn write_tag_index(map: TagMapping, state: CompileState) {
    let tag_map = map.lock().unwrap();
    let ctx = TagIndex {
        tags: tag_map.clone(),
    };
    fs::write("public/tags/index.html", ctx.render(&state)).unwrap();
}

#[inline]
pub fn write_backlinks(map: GlobalBacklinks, state: CompileState) {
    let link_map = map.lock().unwrap();
    let ctx = LinkPage {
        links: link_map.clone(),
    };
    fs::write("public/links/index.html".to_string(), ctx.render(&state)).unwrap();
}

#[inline]
pub fn get_template_file(requested_file: &str) -> Result<String, io::Error> {
    let file_path = format!("templates/{}.html", requested_file);
    if let Ok(filestring) = fs::read_to_string(&file_path) {
        Ok(filestring)
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Could not find {}", requested_file),
        ))
    }
}
