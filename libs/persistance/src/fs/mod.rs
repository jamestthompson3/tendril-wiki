pub mod config;
pub mod utils;

use std::fmt::Write as _;
use std::{
    env, io,
    path::{Path, PathBuf},
};

use chrono::{DateTime, FixedOffset, Local};
use directories::ProjectDirs;
use wikitext::{
    parsers::{parse_meta, NoteHeader, ParsedPages},
    processors::to_template,
};
use render::{index_page::IndexPage, wiki_page::WikiPage, GlobalBacklinks, Render};
use tasks::messages::PatchData;
use tokio::fs::{self, read_to_string};

use thiserror::Error;

use crate::fs::{
    config::read_config,
    utils::{get_file_path, normalize_wiki_location},
};

use self::{
    config::Config,
    utils::{get_archive_file_path, get_archive_location},
};

lazy_static::lazy_static! {
    static ref CONFIG: Config = read_config();
    pub(crate) static ref WIKI_LOCATION: PathBuf = {
        match env::var("TENDRIL_WIKI_DIR") {
            Ok(val) => PathBuf::from(val),
            _ => {
                PathBuf::from(&normalize_wiki_location(&CONFIG.general.wiki_location))
            }
        }
    };
    pub(crate) static ref MEDIA_LOCATION: PathBuf = PathBuf::from(&normalize_wiki_location(&CONFIG.general.media_location));
}

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

pub async fn write_media(filename: &str, bytes: &[u8]) -> Result<(), io::Error> {
    let mut file_path = MEDIA_LOCATION.clone();
    file_path.push(filename);
    fs::write(file_path, bytes).await?;
    Ok(())
}

pub async fn write(data: &PatchData) -> Result<(), WriteWikiError> {
    let current_title_on_disk = if data.old_title != data.title && !data.old_title.is_empty() {
        data.old_title.clone()
    } else {
        // wiki entires are stored by title + .md file ending
        data.title.clone()
    };
    let file_path = get_file_path(&current_title_on_disk).unwrap();
    let mut note_meta = NoteHeader::from(data);
    let now = Local::now().format(DT_FORMAT).to_string();
    // In the case that we're creating a new file
    if !file_path.exists() && data.old_title.is_empty() {
        note_meta.metadata.insert("created".into(), now.clone());
        note_meta.metadata.insert("id".into(), now);
        let note: String = note_meta.into();
        return match fs::write(file_path, note).await {
            Ok(()) => Ok(()),
            Err(e) => {
                eprintln!("Create new file err: {}", e);
                Err(WriteWikiError::WriteError(e))
            }
        };
    }
    note_meta.metadata.insert("modified".into(), now.clone());

    let created = note_meta.metadata.get("created");

    // HACK: Only for legacy notes
    if created.is_none() {
        note_meta.metadata.insert("created".into(), now.clone());
        note_meta.metadata.insert("id".into(), now);
    }
    if note_meta.metadata.get("id").is_none() {
        let created_time = note_meta.metadata.get("created").unwrap().to_owned();
        let parsed_created = match created_time.parse::<DateTime<FixedOffset>>() {
            Ok(dt) => dt.format(DT_FORMAT).to_string(),
            Err(_) => created_time,
        };
        note_meta.metadata.insert("id".into(), parsed_created);
    }
    // END HACK

    let final_note: String = note_meta.into();
    if data.old_title != data.title && !data.old_title.is_empty() {
        let new_location = PathBuf::from(format!(
            "{}{}.md",
            WIKI_LOCATION.to_str().unwrap(),
            data.title
        ));
        // Rename the file to the new title
        match fs::rename(&file_path, &new_location).await {
            Ok(()) => match fs::write(new_location, final_note).await {
                Ok(()) => Ok(()),
                Err(e) => {
                    eprintln!("write renamed file: {}", e);
                    Err(WriteWikiError::WriteError(e))
                }
            },

            Err(e) => Err(WriteWikiError::WriteError(e)),
        }
    } else {
        match fs::write(file_path, final_note).await {
            Ok(()) => Ok(()),
            Err(e) => Err(WriteWikiError::WriteError(e)),
        }
    }
}

pub async fn delete(requested_file: &str) -> Result<(), io::Error> {
    let file_path = get_file_path(requested_file).unwrap();
    if !file_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Could not find requested file",
        ));
    }
    fs::remove_file(file_path).await?;
    Ok(())
}

pub async fn read(
    requested_file: String,
    backlinks: GlobalBacklinks,
) -> Result<String, ReadPageError> {
    let file_path = get_file_path(&requested_file)?;
    match path_to_data_structure(&file_path).await {
        Ok(note) => {
            let templatted = to_template(&note);
            let link_vals = backlinks.lock().await;
            let links = link_vals.get(&templatted.page.title);
            let output = WikiPage::new(&templatted.page, links).render().await;
            Ok(output)
        }
        Err(e) => Err(e),
    }
}

pub async fn write_entries(pages: &ParsedPages, backlinks: &GlobalBacklinks) {
    let page_vals = pages.lock().await;
    let link_vals = backlinks.lock().await;
    for page in page_vals.iter() {
        let links = link_vals.get(&page.title);
        let output = WikiPage::new(page, links).render().await;
        let formatted_title = page.title.replace('/', "-");
        let out_dir = format!("public/{}", formatted_title);
        // TODO use path here instead of title? Since `/` in title can cause issues in fs::write
        fs::create_dir(&out_dir)
            .await
            .unwrap_or_else(|e| eprintln!("{:?}\nCould not create dir: {}", e, out_dir));
        let out_file = format!("public/{}/index.html", formatted_title);
        fs::write(&out_file, output)
            .await
            .unwrap_or_else(|e| eprintln!("{:?}\nCould not write file: {}", e, out_file));
    }
}

pub async fn write_index_page(user: String, host: String) {
    let ctx = IndexPage { user, host };
    fs::write("public/index.html", ctx.render().await)
        .await
        .unwrap();
}

pub async fn read_note_cache() -> String {
    let project_dir = ProjectDirs::from("", "", "tendril").unwrap();
    let mut data_dir = project_dir.data_dir().to_owned();
    data_dir.push("note_cache");
    read_to_string(&data_dir).await.unwrap()
}

pub async fn write_note_cache(cache: String) {
    let project_dir = ProjectDirs::from("", "", "tendril").unwrap();
    let mut data_dir = project_dir.data_dir().to_owned();
    data_dir.push("note_cache");
    fs::write(data_dir, cache).await.unwrap();
}

pub async fn create_journal_entry(entry: String) -> Result<PatchData, std::io::Error> {
    let now = Local::now();
    let daily_file = now.format("%Y-%m-%d").to_string();
    let path = get_file_path(&daily_file).unwrap();
    if path.exists() {
        let mut entry_file = read_to_string(&path).await.unwrap();
        write!(entry_file, "\n\n[{}] {}", now.format("%H:%M"), entry).unwrap();
        println!("\x1b[38;5;47mdaily journal updated\x1b[0m");
        fs::write(path, &entry_file).await?;
        Ok(NoteHeader::from(entry_file).into())
    } else {
        let docstring = format!(
            r#"
---
title: {}
tags: [daily notes]
created: {:?}
---
[{}] {}
"#,
            daily_file,
            now,
            now.format("%H:%M"),
            entry
        );
        println!("\x1b[38;5;47mdaily journal updated\x1b[0m");
        fs::write(get_file_path(&daily_file).unwrap(), docstring.clone()).await?;
        Ok(NoteHeader::from(docstring).into())
    }
}

pub async fn write_archive(compressed: Vec<u8>, title: &str) {
    let location = get_archive_file_path(title);
    fs::write(location, compressed).await.unwrap();
}

pub async fn move_archive(old_title: String, new_title: String) {
    let archive = get_archive_location();
    let old_location = archive.join(old_title);
    let new_location = archive.join(new_title);
    fs::rename(old_location, new_location).await.unwrap();
}

// TODO: this is really dependent on file system ops, won't be good if we change the storage
// backend.
pub async fn path_to_string<P: AsRef<Path> + ?Sized>(path: &P) -> Result<String, std::io::Error> {
    read_to_string(&path).await
}

pub async fn path_to_data_structure(path: &Path) -> Result<NoteHeader, ReadPageError> {
    match path_to_string(path).await {
        Ok(reader) => {
            let lines = reader.lines();
            let meta = parse_meta(lines, path.to_str().unwrap());
            Ok(meta)
        }
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => Err(ReadPageError::PageNotFoundError),
            e => {
                println!("___ {:?}", e);
                Err(ReadPageError::DeserializationError)
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::fs::utils::parse_location;

    use std::{env, path::PathBuf};

    #[test]
    fn formats_wiki_location() {
        assert_eq!(parse_location("./wiki"), PathBuf::from("./wiki/"));
        env::set_var("HOME", "test");
        assert_eq!(parse_location("~/wiki"), PathBuf::from("test/wiki/"));
        assert_eq!(
            parse_location("/user/~/wiki"),
            PathBuf::from("/user/test/wiki/")
        );
    }
}
