use std::{
    collections::HashMap,
    fs,
    sync::{Arc, Mutex},
};

use sailfish::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "main.stpl")]
pub struct BasicPage<'a> {
    title: &'a String,
    body: &'a String,
    tags: &'a Vec<String>,
    backlinks: Vec<String>,
}

#[derive(TemplateOnce)]
#[template(path = "tags.stpl")]
struct TagPage {
    title: String,
    tags: Vec<String>,
}

#[derive(TemplateOnce)]
#[template(path = "backlinks.stpl")]
struct LinkPage {
    links: HashMap<String, Vec<String>>,
}

#[derive(Debug)]
pub struct TemplattedPage {
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
}

pub struct ParsedTemplate {
    pub outlinks: Vec<String>,
    pub page: TemplattedPage,
}

pub type TagMapping = Arc<Mutex<HashMap<String, Vec<String>>>>;
pub type GlobalBacklinks = Arc<Mutex<HashMap<String, Vec<String>>>>;
pub type ParsedPages = Arc<Mutex<Vec<TemplattedPage>>>;

pub fn render_template(page: &TemplattedPage, links: Option<&Vec<String>>) -> String {
    let mut backlinks = match links {
        Some(links) => links.to_owned(),
        None => Vec::new(),
    };
    backlinks.dedup();
    let ctx = BasicPage {
        title: &page.title,
        tags: &page.tags,
        body: &page.body,
        backlinks,
    };
    ctx.render_once().unwrap()
}

pub fn write_entries(pages: &ParsedPages, backlinks: &GlobalBacklinks) {
    let page_vals = pages.lock().unwrap();
    let link_vals = backlinks.lock().unwrap();
    for page in page_vals.iter() {
        let links = link_vals.get(&page.title);
        let output = render_template(&page, links);
        // TODO use path here instead of title? Since `/` in title can cause issues in fs::write
        fs::write(
            format!("public/{}.html", page.title.replace('/', "-")),
            output,
        )
        .unwrap();
    }
}

pub fn write_tag_pages(map: TagMapping) {
    let tag_map = map.lock().unwrap();
    for key in tag_map.keys() {
        let title = key.to_string();
        let tags = tag_map.get(key).unwrap().to_owned();
        let ctx = TagPage {
            title: title.clone(),
            tags,
        };
        fs::write(
            format!("public/tags/{}.html", title),
            ctx.render_once().unwrap(),
        )
        .unwrap();
    }
}

pub fn write_backlinks(map: GlobalBacklinks) {
    let link_map = map.lock().unwrap();
    let ctx = LinkPage {
        links: link_map.clone(),
    };
    fs::write(
        format!("public/links/index.html"),
        ctx.render_once().unwrap(),
    )
    .unwrap();
}
