use crate::parsers::{to_html, NoteMeta, ParsedPages, ParsedTemplate, TemplattedPage};

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
