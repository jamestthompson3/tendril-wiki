use std::{fs, path::PathBuf};

use crate::{parsers::NoteMeta, processors::tags::TagsArray};
use crate::{
    parsers::{
        parse_meta, parse_wiki_entry, path_to_data_structure, path_to_reader, render_template,
        GlobalBacklinks, TagMapping,
    },
    processors::to_template,
};
use urlencoding::decode;

use super::WebFormData;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WriteWikiError {
    #[error("title cannot be changed")]
    TitleInvalid,
    #[error("could not write updated data to file")]
    WriteError(std::io::Error),
    #[error("unknown write error")]
    Unknown,
}

pub fn write(wiki_location: &String, data: WebFormData) -> Result<(), WriteWikiError> {
    let mut file_location = PathBuf::from(parse_wiki_entry(wiki_location));
    // wiki entires are stored by title + .md file ending
    let mut title_location = data.title.clone();
    title_location.push_str(".md");
    file_location.push(&title_location);
    // In the case that we're creating a new file
    if !file_location.exists() {
        let note_meta = NoteMeta::from(data);
        let note: String = note_meta.into();
        return match fs::write(file_location, note) {
            Ok(()) => Ok(()),
            Err(e) => Err(WriteWikiError::WriteError(e)),
        };
    }
    let mut note_meta = parse_meta(
        path_to_reader(&file_location).unwrap(),
        file_location.to_str().unwrap(),
    );
    if *note_meta.metadata.get("title").unwrap() != data.title {
        // add relinking later, otherwise other wiki links will be borked
        return Err(WriteWikiError::TitleInvalid);
    }
    // Some reason the browser adds \r\n
    note_meta.content = data.body.replace("\r\n", "\n");
    let updated_tags: TagsArray = data.tags.into();

    if let Some(existing_tags) = note_meta.metadata.get("tags") {
        let existing_tag_array = TagsArray::from(existing_tags.to_string());
        if updated_tags.len() != existing_tag_array.len() {
            note_meta
                .metadata
                .insert("tags".into(), updated_tags.write());
        }
    } else if updated_tags.len() > 0 {
        note_meta
            .metadata
            .insert("tags".to_string(), updated_tags.write());
    }

    let final_note: String = note_meta.into();
    match fs::write(file_location, final_note) {
        Ok(()) => Ok(()),
        Err(e) => Err(WriteWikiError::WriteError(e)),
    }
}

pub fn read(
    wiki_location: &String,
    mut requested_file: String,
    _tags: TagMapping,
    backlinks: GlobalBacklinks,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut file_location = PathBuf::from(parse_wiki_entry(wiki_location));
    requested_file = decode(&requested_file)?;
    requested_file.push_str(".md");
    file_location.push(requested_file);
    let note = path_to_data_structure(&file_location)?;
    let templatted = to_template(&note);
    let link_vals = backlinks.lock().unwrap();
    let links = link_vals.get(&templatted.page.title);
    let output = render_template(&templatted.page, links);
    Ok(output)
}
