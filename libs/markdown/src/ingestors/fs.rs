use std::{fs, path::PathBuf};

use crate::parsers::{parse_meta, parse_wiki_entry, path_to_reader};
use crate::processors::process_tags;

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

pub fn write(wiki_location: String, data: WebFormData) -> Result<(), WriteWikiError> {
    let mut file_location = PathBuf::from(parse_wiki_entry(&wiki_location));
    // wiki entires are stored by title + .md file ending
    let mut title_location = data.title.clone();
    title_location.push_str(".md");
    file_location.push(&title_location);
    let mut note_meta = parse_meta(
        path_to_reader(&file_location),
        file_location.to_str().unwrap(),
    );
    if *note_meta.metadata.get("title").unwrap() != data.title {
        // add relinking later, otherwise other wiki links will be borked
        return Err(WriteWikiError::TitleInvalid);
    }
    note_meta.content = data.body.replace("\r\n", "\n");
    let updated_tags = data
        .tags
        .iter()
        .filter(|t| !t.is_empty())
        .map(|t| t.to_string())
        .collect::<String>();

    if let Some(existing_tags) = note_meta.metadata.get("tags") {
        if updated_tags.len() != process_tags(&existing_tags).len() {
            note_meta
                .metadata
                .insert("tags".to_string(), format!("{:?}", updated_tags));
        }
    } else if updated_tags.len() > 0 {
        note_meta
            .metadata
            .insert("tags".to_string(), format!("{:?}", updated_tags));
    }

    let final_note: String = note_meta.into();
    match fs::write(file_location, final_note) {
        Ok(()) => Ok(()),
        Err(e) => Err(WriteWikiError::WriteError(e)),
    }
}
