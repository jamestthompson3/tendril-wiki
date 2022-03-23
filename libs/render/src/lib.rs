use std::{
    collections::BTreeMap,
    env, fs, io,
    sync::{Arc, Mutex},
};

#[macro_use]
extern crate lazy_static;

#[cfg(not(debug_assertions))]
use directories::ProjectDirs;

use link_page::LinkPage;
use markdown::parsers::TemplattedPage;

pub mod file_upload_page;
pub mod help_page;
pub mod index_page;
pub mod link_page;
pub mod login_page;
pub mod new_page;
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

pub trait Render {
    fn render(&self) -> String;
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

#[inline]
fn process_included_file(file: String, page: Option<&TemplattedPage>) -> String {
    let state = if STATIC_BUILD.as_str() == "true" {
        CompileState::Static
    } else {
        CompileState::Dynamic
    };
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
        "meta" => {
            let templatefile = get_template_file("meta").unwrap();
            let page = page.unwrap();
            let icon_path = match &page.metadata.get("icon") {
                Some(icon) => format!("/files/{}", icon),
                None => String::from("static/favicon.ico"),
            };
            let desc = if page.raw_md.len() >= 100 {
                let mut shortened_desc = page.raw_md.clone();
                shortened_desc.truncate(80);
                shortened_desc.push_str("...");
                shortened_desc
            } else {
                page.raw_md.clone()
            };
            templatefile
                .replace("<%= title %>", &page.title)
                .replace("<%= desc %>", &desc)
                .replace("<%= icon %>", &icon_path)
        }
        "footer" => get_template_file("footer").unwrap(),
        "search_form" => get_template_file("search_form").unwrap(),
        _ => String::with_capacity(0),
    }
}

pub fn render_includes(ctx: String, page: Option<&TemplattedPage>) -> String {
    let lines = ctx.lines().map(|line| {
        if line.contains("<%= include") {
            process_included_file(parse_includes(line), page)
        } else {
            line.to_string()
        }
    });
    lines.collect::<Vec<String>>().join("\n")
}

#[inline]
pub fn write_backlinks(map: GlobalBacklinks) {
    let link_map = map.lock().unwrap();
    let ctx = LinkPage {
        links: link_map.clone(),
    };
    fs::write("public/links/index.html", ctx.render()).unwrap();
}

#[inline]
pub fn get_template_file(requested_file: &str) -> Result<String, io::Error> {
    let file_path = get_template_location(requested_file);
    if let Ok(filestring) = fs::read_to_string(&file_path) {
        Ok(filestring)
    } else {
        eprintln!("Could not find {}", requested_file);
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Could not find {}", requested_file),
        ))
    }
}

#[cfg(debug_assertions)]
fn get_template_location(requested_file: &str) -> String {
    format!("templates/{}.html", requested_file)
}

#[cfg(not(debug_assertions))]
fn get_template_location(requested_file: &str) -> String {
    let project_dir = ProjectDirs::from("", "", "tendril").unwrap();
    let mut data_dir = project_dir.data_dir().to_owned();
    data_dir.push(format!("templates/{}.html", requested_file));
    data_dir.to_string_lossy().into()
}
