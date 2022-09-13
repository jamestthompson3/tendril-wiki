use crate::parsers::{to_html, Html, Note, ParsedPages, ParsedTemplate, TemplattedPage};

pub mod tags;

use self::tags::*;

pub fn to_template(note: &Note) -> ParsedTemplate {
    let content_type = if let Some(content_type) = note.header.get("type") {
        content_type.as_str()
    } else {
        "text"
    };
    let html = if content_type == "html" {
        Html {
            body: note.content.clone(),
            outlinks: Vec::with_capacity(0),
        }
    } else {
        to_html(&note.content)
    };
    let default_title = "Untitled".to_string();
    let title = note
        .header
        .get("title")
        .unwrap_or(&default_title)
        .to_owned();
    let tags = match note.header.get("tags") {
        None => Vec::with_capacity(0),
        Some(raw_tags) => TagsArray::new(raw_tags).values,
    };
    let mut rendered_metadata = note.header.to_owned();
    // We're already showing this, so no need to dump it in the table...
    rendered_metadata.remove("title");
    rendered_metadata.remove("tags");
    let desc = if note.content.len() >= 100 {
        if content_type != "html" {
            let mut shortened_desc = note.content.clone();
            shortened_desc.truncate(80);
            shortened_desc.push_str("...");
            shortened_desc
        } else {
            title.clone()
        }
    } else {
        note.content.clone()
    };
    let page = TemplattedPage {
        title,
        tags,
        body: html.body,
        metadata: rendered_metadata,
        desc,
    };
    ParsedTemplate {
        outlinks: html.outlinks,
        page,
    }
}

pub async fn update_templatted_pages(page: TemplattedPage, pages: ParsedPages) {
    let mut tempatted_pages = pages.lock().await;
    tempatted_pages.push(page);
}
