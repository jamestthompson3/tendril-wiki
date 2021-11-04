use std::{fs, io, path::PathBuf};

use chrono::Local;
use markdown::{
    parsers::{
        parse_meta, path_to_data_structure, EditPageData, GlobalBacklinks, NoteMeta, ParsedPages,
        TagMapping,
    },
    processors::{tags::TagsArray, to_template},
};
use render::{index_page::IndexPage, wiki_page::WikiPage, Render};
use tasks::{path_to_reader, CompileState};
use urlencoding::decode;

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

pub fn write_media(file_location: &str, bytes: &[u8]) -> Result<(), io::Error> {
    fs::write(file_location, bytes)?;
    Ok(())
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
        let mut note_meta = NoteMeta::from(data);
        note_meta
            .metadata
            .insert("created".into(), Local::now().to_string());
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
    // FIXME: relying on the metadata title attribute when making the template when
    // the path is used everywhere else isn't great...
    note_meta.metadata = data.metadata;
    note_meta
        .metadata
        .insert("title".into(), data.title.clone());
    // Update last edited time
    note_meta
        .metadata
        .insert("modified".into(), Local::now().to_string());

    let updated_tags: TagsArray = data.tags.into();

    note_meta
        .metadata
        .insert("tags".into(), updated_tags.write());

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

pub fn delete(wiki_location: &str, requested_file: &str) -> Result<(), io::Error> {
    let mut file_location = String::from(wiki_location);
    if let Ok(mut file) = decode(requested_file) {
        file.push_str(".md");
        file_location.push_str(&file);
        let file_path = PathBuf::from(file_location);
        if !file_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Could not find reuqested file",
            ));
        }
        fs::remove_file(file_path)?;
        Ok(())
    } else {
        Err(std::io::Error::new(
            io::ErrorKind::InvalidInput,
            "Could not decode file name",
        ))
    }
}

pub fn read(
    wiki_location: &str,
    requested_file: String,
    _tags: TagMapping,
    backlinks: GlobalBacklinks,
) -> Result<String, ReadPageError> {
    let file_path = get_file_path(wiki_location, &requested_file)?;
    if let Ok(note) = path_to_data_structure(&file_path) {
        let templatted = to_template(&note);
        let link_vals = backlinks.lock().unwrap();
        let links = link_vals.get(&templatted.page.title);
        let output = WikiPage::new(&templatted.page, links).render(&CompileState::Dynamic); // we have a hard coded type since this is only called on the web server
        Ok(output)
    } else {
        Err(ReadPageError::DeserializationError)
    }
}

pub fn get_file_path(wiki_location: &str, requested_file: &str) -> Result<PathBuf, ReadPageError> {
    let mut file_location = String::from(wiki_location);
    if let Ok(mut file) = decode(requested_file) {
        file.push_str(".md");
        file_location.push_str(&file);
        let file_path = PathBuf::from(file_location);
        if !file_path.exists() {
            return Err(ReadPageError::PageNotFoundError);
        }
        Ok(file_path)
    } else {
        Err(ReadPageError::DecodeError)
    }
}

#[inline]
pub fn write_entries(pages: &ParsedPages, backlinks: &GlobalBacklinks, state: CompileState) {
    let page_vals = pages.lock().unwrap();
    let link_vals = backlinks.lock().unwrap();
    for page in page_vals.iter() {
        let links = link_vals.get(&page.title);
        let output = WikiPage::new(page, links).render(&state);
        // TODO use path here instead of title? Since `/` in title can cause issues in fs::write
        fs::create_dir(format!("public/{}", page.title.replace('/', "-"))).unwrap();
        fs::write(
            format!("public/{}/index.html", page.title.replace('/', "-")),
            output,
        )
        .unwrap();
    }
}

#[inline]
pub fn write_index_page(user: String, state: CompileState) {
    let ctx = IndexPage { user };
    fs::write("public/index.html", ctx.render(&state)).unwrap();
}
