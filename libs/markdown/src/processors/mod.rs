use crate::parsers::{
    to_html, GlobalBacklinks, NoteMeta, ParsedPages, ParsedTemplate, TagMapping, TemplattedPage,
};

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
        Some(raw_tags) => process_tags(raw_tags),
    };
    let page = TemplattedPage {
        title,
        tags,
        body: html.body,
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

pub fn update_tag_map(title: &str, tags: &Vec<String>, tag_map: TagMapping) {
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

// TODO:
// Eventually it would be nice to properly serialize note meta props so we don't have to parse.
pub fn process_tags(tag_str: &str) -> Vec<String> {
    if tag_str.find('[') != None {
        let split_tags = tag_str
            .strip_prefix('[')
            .unwrap()
            .strip_suffix(']')
            .unwrap()
            .split(',')
            .filter(|s| !s.is_empty() && s != &" ") // maybe use filter_map here?
            .map(|s| s.trim())
            .map(|s| s.to_owned())
            .collect();
        return split_tags;
    }
    tag_str
        .split(' ')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_owned())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tags_with_wikilink() {
        let tag_string = "[reality building, Article]";
        assert_eq!(
            process_tags(tag_string),
            vec!["reality building", "Article"]
        );
    }

    #[test]
    fn parse_tags_without_wikilinks() {
        let tag_string = "Tools Article project-management";
        assert_eq!(
            process_tags(tag_string),
            vec!["Tools", "Article", "project-management"]
        );
    }
}
