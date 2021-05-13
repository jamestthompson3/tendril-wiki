use std::{fs, path::PathBuf};

use crate::{parsers::NoteMeta, processors::tags::TagsArray};
use crate::{
    parsers::{
        parse_meta, path_to_data_structure, path_to_reader, render_template, GlobalBacklinks,
        TagMapping,
    },
    processors::to_template,
};
use urlencoding::decode;

use super::EditPageData;
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

#[derive(Error, Debug)]
pub enum ReadPageError {
    #[error("Could not decode page name")]
    DecodeError,
    #[error("could not deserialize page")]
    DeserializationError,
    #[error("could not find page")]
    PageNotFoundError,
    #[error("unknown read error")]
    Unknown,
}

pub fn write(
    wiki_location: &str,
    data: EditPageData,
    backlinks: GlobalBacklinks,
) -> Result<(), WriteWikiError> {
    let mut file_location = String::from(wiki_location);
    let mut title_location: String;
    if data.old_title != data.title && !data.old_title.is_empty() {
        title_location = data.old_title.clone();
    } else {
        // wiki entires are stored by title + .md file ending
        title_location = data.title.clone();
    }
    title_location.push_str(".md");
    file_location.push_str(&title_location);
    // In the case that we're creating a new file
    if !PathBuf::from(&file_location).exists() {
        let note_meta = NoteMeta::from(data);
        let note: String = note_meta.into();
        return match fs::write(file_location, note) {
            Ok(()) => Ok(()),
            Err(e) => {
                eprintln!("Create new file err: {}", e);
                Err(WriteWikiError::WriteError(e))
            }
        };
    }
    let mut note_meta = parse_meta(path_to_reader(&file_location).unwrap(), &file_location);
    // Some reason the browser adds \r\n
    note_meta.content = data.body.replace("\r\n", "\n");
    let updated_tags: TagsArray = data.tags.into();

    note_meta
        .metadata
        .insert("tags".into(), updated_tags.write());

    if data.old_title != data.title && !data.old_title.is_empty() {
        note_meta
            .metadata
            .insert("title".into(), data.title.clone());
    }

    let final_note: String = note_meta.into();
    if data.old_title != data.title && !data.old_title.is_empty() {
        // Relink all pages that reference this page
        let links = backlinks.lock().unwrap();
        let linked_pages = links.get(&data.old_title);
        if let Some(linked_pages) = linked_pages {
            for page in linked_pages {
                let mut wiki_loc = String::from(wiki_location);
                let mut page = page.clone();
                page.push_str(".md");
                wiki_loc.push_str(&page);
                let raw_page = fs::read_to_string(&wiki_loc).unwrap();
                let relinked_page = raw_page.replace(&data.old_title, &data.title);
                fs::write(wiki_loc, relinked_page).unwrap();
            }
        }
        let new_location = file_location.replace(&data.old_title, &data.title);
        // Rename the file to the new title
        match fs::rename(&file_location, &new_location) {
            Ok(()) => match fs::write(new_location, final_note) {
                Ok(()) => Ok(()),
                Err(e) => {
                    eprintln!("write renamed file: {}", e);
                    Err(WriteWikiError::WriteError(e))
                }
            },
            Err(e) => Err(WriteWikiError::WriteError(e)),
        }
    } else {
        match fs::write(file_location, final_note) {
            Ok(()) => Ok(()),
            Err(e) => Err(WriteWikiError::WriteError(e)),
        }
    }
}

pub fn read(
    wiki_location: &str,
    requested_file: String,
    _tags: TagMapping,
    backlinks: GlobalBacklinks,
) -> Result<String, ReadPageError> {
    let mut file_location = String::from(wiki_location);
    if let Ok(mut file) = decode(&requested_file) {
        file.push_str(".md");
        file_location.push_str(&file);
        let file_path = PathBuf::from(file_location);
        if !file_path.exists() {
            return Err(ReadPageError::PageNotFoundError);
        }
        if let Ok(note) = path_to_data_structure(&file_path) {
            let templatted = to_template(&note);
            let link_vals = backlinks.lock().unwrap();
            let links = link_vals.get(&templatted.page.title);
            let output = render_template(&templatted.page, links);
            Ok(output)
        } else {
            Err(ReadPageError::DeserializationError)
        }
    } else {
        Err(ReadPageError::DecodeError)
    }
}
