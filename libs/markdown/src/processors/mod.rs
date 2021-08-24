use crate::parsers::{
    to_html, GlobalBacklinks, NoteMeta, ParsedPages, ParsedTemplate, TagMapping, TemplattedPage,
};

pub mod tags;

use self::tags::*;

pub fn to_template(note: &NoteMeta) -> ParsedTemplate {
    let html = to_html(&note.content);
    let default_title = "Untitled".to_string();
    let title = note
        .metadata
        .get("title")
        .unwrap_or(&default_title)
        .to_owned();
    let tags = match note.metadata.get("tags") {
        None => Vec::with_capacity(0),
        Some(raw_tags) => TagsArray::new(raw_tags).values,
    };
    let mut rendered_metadata = note.metadata.to_owned();
    // We're already showing this, so no need to dump it in the table...
    rendered_metadata.remove("title");
    rendered_metadata.remove("tags");
    let page = TemplattedPage {
        title,
        tags,
        body: html.body,
        metadata: rendered_metadata,
        raw_md: note.content.clone(),
    };
    ParsedTemplate {
        outlinks: html.outlinks,
        page,
    }
}

pub fn update_templatted_pages(page: TemplattedPage, pages: ParsedPages) {
    let mut tempatted_pages = pages.lock().unwrap();
    tempatted_pages.push(page);
}

pub fn update_backlinks(title: &str, outlinks: &[String], backlinks: GlobalBacklinks) {
    let mut global_backlinks = backlinks.lock().unwrap();
    for link in outlinks.iter() {
        match global_backlinks.get(&link.to_string()) {
            Some(links) => {
                let mut updated_links = links.to_owned();
                updated_links.push(title.to_owned());
                global_backlinks.insert(link.to_string(), updated_links);
            }
            None => {
                global_backlinks.insert(link.to_string(), vec![title.to_owned()]);
            }
        }
    }
}

pub fn update_tag_map(title: &str, tags: &[String], tag_map: TagMapping) {
    let mut global_tag_map = tag_map.lock().unwrap();
    for tag in tags.iter() {
        match global_tag_map.get(&tag.to_string()) {
            Some(tags) => {
                let mut updated_tags = tags.to_owned();
                updated_tags.push(title.to_owned());
                global_tag_map.insert(tag.to_string(), updated_tags);
            }
            None => {
                global_tag_map.insert(tag.to_string(), vec![title.to_owned()]);
            }
        }
    }
}
