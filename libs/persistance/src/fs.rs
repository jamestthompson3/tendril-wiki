use std::{fs, io, path::PathBuf};

use chrono::{DateTime, FixedOffset, Local};
use directories::ProjectDirs;
use markdown::{
    parsers::{parse_meta, path_to_data_structure, EditPageData, NoteMeta, ParsedPages},
    processors::{tags::TagsArray, to_template},
};
use render::{index_page::IndexPage, wiki_page::WikiPage, GlobalBacklinks, Render};
use tasks::path_to_reader;
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

const DT_FORMAT: &str = "%Y%m%d%H%M%S";

pub fn write_media(file_location: &str, bytes: &[u8]) -> Result<(), io::Error> {
    fs::write(file_location, bytes)?;
    Ok(())
}

pub fn write(wiki_location: &str, data: EditPageData) -> Result<(), WriteWikiError> {
    let mut file_location = String::from(wiki_location);
    let mut title_location = if data.old_title != data.title && !data.old_title.is_empty() {
        data.old_title.clone()
    } else {
        // wiki entires are stored by title + .md file ending
        data.title.clone()
    };
    title_location.push_str(".md");
    file_location.push_str(&title_location);
    // In the case that we're creating a new file
    if !PathBuf::from(&file_location).exists() {
        let mut note_meta = NoteMeta::from(data);
        let now = Local::now().format(DT_FORMAT).to_string();
        note_meta.metadata.insert("created".into(), now.clone());
        note_meta.metadata.insert("id".into(), now);
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
    let now = Local::now().format(DT_FORMAT).to_string();
    note_meta
        .metadata
        .insert("title".into(), data.title.clone());
    // Update last edited time
    note_meta.metadata.insert("modified".into(), now.clone());

    let created = note_meta.metadata.get("created");

    if created.is_none() {
        note_meta.metadata.insert("created".into(), now.clone());
        note_meta.metadata.insert("id".into(), now);
    }
    if note_meta.metadata.get("id").is_none() {
        let created_time = note_meta.metadata.get("created").unwrap().to_owned();
        note_meta.metadata.insert(
            "id".into(),
            created_time
                .parse::<DateTime<FixedOffset>>()
                .unwrap()
                .format(DT_FORMAT)
                .to_string(),
        );
    }
    let updated_tags: TagsArray = data.tags.into();

    note_meta
        .metadata
        .insert("tags".into(), updated_tags.write());

    let final_note: String = note_meta.into();
    if data.old_title != data.title && !data.old_title.is_empty() {
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
        file.to_mut().push_str(".md");
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
    backlinks: GlobalBacklinks,
) -> Result<String, ReadPageError> {
    let file_path = get_file_path(wiki_location, &requested_file)?;
    if let Ok(note) = path_to_data_structure(&file_path) {
        let templatted = to_template(&note);
        let link_vals = backlinks.lock().unwrap();
        let links = link_vals.get(&templatted.page.title);
        let output = WikiPage::new(&templatted.page, links).render(); // we have a hard coded type since this is only called on the web server
        Ok(output)
    } else {
        Err(ReadPageError::DeserializationError)
    }
}

/// Returns the PathBuf if an entry exists, returns an error if the file isn't found or it couldn't
/// parse the location.
pub fn get_file_path(wiki_location: &str, requested_file: &str) -> Result<PathBuf, ReadPageError> {
    let mut file_location = String::from(wiki_location);
    if let Ok(mut file) = decode(requested_file) {
        file.to_mut().push_str(".md");
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
pub fn write_entries(pages: &ParsedPages, backlinks: &GlobalBacklinks) {
    let page_vals = pages.lock().unwrap();
    let link_vals = backlinks.lock().unwrap();
    for page in page_vals.iter() {
        let links = link_vals.get(&page.title);
        let output = WikiPage::new(page, links).render();
        let formatted_title = page.title.replace('/', "-");
        let out_dir = format!("public/{}", formatted_title);
        // TODO use path here instead of title? Since `/` in title can cause issues in fs::write
        fs::create_dir(&out_dir)
            .unwrap_or_else(|e| eprintln!("{:?}\nCould not create dir: {}", e, out_dir));
        let out_file = format!("public/{}/index.html", formatted_title);
        fs::write(&out_file, output)
            .unwrap_or_else(|e| eprintln!("{:?}\nCould not write file: {}", e, out_file));
    }
}

#[inline]
pub fn write_index_page(user: String) {
    let ctx = IndexPage { user };
    fs::write("public/index.html", ctx.render()).unwrap();
}

#[inline]
pub fn read_note_cache() -> String {
    let project_dir = ProjectDirs::from("", "", "tendril").unwrap();
    let mut data_dir = project_dir.data_dir().to_owned();
    data_dir.push("note_cache");
    std::fs::read_to_string(&data_dir).unwrap()
}

#[inline]
pub fn write_note_cache(cache: String) {
    let project_dir = ProjectDirs::from("", "", "tendril").unwrap();
    let mut data_dir = project_dir.data_dir().to_owned();
    data_dir.push("note_cache");
    std::fs::write(data_dir, cache).unwrap();
}
