use std::{
    collections::HashMap,
    fs,
    sync::{Arc, Mutex},
};

use super::html::{format_links, to_html};
use super::meta::NoteMeta;

use sailfish::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "main.stpl")]
pub struct BasicPage<'a> {
    title: &'a String,
    body: &'a String,
    tags: &'a Vec<&'a str>,
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

pub struct TemplattedPage<'a> {
    pub title: String,
    pub body: String,
    pub tags: Vec<&'a str>,
}

pub struct ParsedTemplate<'a> {
    pub outlinks: Vec<String>,
    pub page: TemplattedPage<'a>,
}

pub type TagMapping = Arc<Mutex<HashMap<String, Vec<String>>>>;
pub type GlobalBacklinks = Arc<Mutex<HashMap<String, Vec<String>>>>;

pub fn template(note: &NoteMeta) -> ParsedTemplate {
    let html = to_html(&note.content);
    let default_title = "Untitled".to_string();
    let title = note
        .metadata
        .get("title")
        .unwrap_or(&default_title)
        .to_owned();
    let tags = match note.metadata.get("tags") {
        None => Vec::with_capacity(0),
        Some(raw_tags) => parse_tags(raw_tags),
    };
    let page = TemplattedPage {
        title,
        tags,
        body: html.body,
    };
    ParsedTemplate {
        outlinks: html.outlinks,
        page,
    }
}

pub fn render_template(page: &TemplattedPage) -> String {
    let ctx = BasicPage {
        title: &page.title,
        tags: &page.tags,
        body: &page.body,
    };
    ctx.render_once().unwrap()
}

pub fn update_backlinks(title: &str, outlinks: &Vec<String>, backlinks: GlobalBacklinks) {
    let mut global_backlinks = backlinks.lock().unwrap();
    for link in outlinks.iter() {
        match global_backlinks.get(&link.to_string()) {
            Some(links) => {
                // TODO: Let's not allocate so much
                let mut updated_links = links.clone();
                updated_links.push(title.to_owned());
                global_backlinks.insert(link.to_string(), updated_links);
            }
            None => {
                global_backlinks.insert(link.to_string(), vec![title.to_owned()]);
            }
        }
    }
}

pub fn update_tag_map(title: &str, tags: &Vec<&str>, tag_map: TagMapping) {
    let mut global_tag_map = tag_map.lock().unwrap();
    for tag in tags.iter() {
        match global_tag_map.get(&tag.to_string()) {
            Some(tags) => {
                // TODO: Let's not allocate so much
                let mut updated_tags = tags.clone();
                updated_tags.push(title.to_owned());
                global_tag_map.insert(tag.to_string(), updated_tags);
            }
            None => {
                global_tag_map.insert(tag.to_string(), vec![title.to_owned()]);
            }
        }
    }
}

pub fn template_tag_pages(map: TagMapping) {
    let tag_map = map.lock().unwrap();
    for key in tag_map.keys() {
        let title = key.to_string();
        let tags = tag_map
            .get(key)
            .unwrap()
            .iter()
            .map(|tag| format_links(tag))
            .collect::<Vec<String>>()
            .to_owned();
        let ctx = TagPage {
            title: title.clone(),
            tags,
        };
        fs::write(
            format!("public/tags/{}.html", title.replace(' ', "_")),
            ctx.render_once().unwrap(),
        )
        .unwrap();
    }
}

pub fn template_backlinks(map: GlobalBacklinks) {
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

// TODO:
// Eventually it would be nice to properly serialize note meta props so we don't have to parse.
pub fn parse_tags(tag_str: &str) -> Vec<&str> {
    if tag_str.find('[') != None {
        let split_tags = tag_str
            .strip_prefix('[')
            .unwrap()
            .strip_suffix(']')
            .unwrap()
            .split(',')
            .filter(|s| !s.is_empty() && s != &" ") // maybe use filter_map here?
            .map(|s| s.trim())
            .collect();
        return split_tags;
    }
    tag_str.split(' ').filter(|s| !s.is_empty()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tags_with_wikilink() {
        let tag_string = "[reality building, Article]";
        assert_eq!(parse_tags(tag_string), vec!["reality building", "Article"]);
    }

    #[test]
    fn parse_tags_without_wikilinks() {
        let tag_string = "Tools Article project-management";
        assert_eq!(
            parse_tags(tag_string),
            vec!["Tools", "Article", "project-management"]
        );
    }
}
