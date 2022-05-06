use std::{
    io,
    path::{Path, PathBuf, MAIN_SEPARATOR},
};

use chrono::{DateTime, FixedOffset, Local};
use directories::{ProjectDirs, UserDirs};
use markdown::{
    parsers::{parse_meta, NoteMeta, ParsedPages},
    processors::to_template,
};
use render::{index_page::IndexPage, wiki_page::WikiPage, GlobalBacklinks, Render};
use tasks::messages::PatchData;
use tokio::fs::{self, read_to_string};
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

pub async fn write_media(file_location: &str, bytes: &[u8]) -> Result<(), io::Error> {
    fs::write(file_location, bytes).await?;
    Ok(())
}

pub async fn write(wiki_location: &str, data: &PatchData) -> Result<(), WriteWikiError> {
    let mut file_location = String::from(wiki_location);
    let mut title_location = if data.old_title != data.title && !data.old_title.is_empty() {
        data.old_title.clone()
    } else {
        // wiki entires are stored by title + .md file ending
        data.title.clone()
    };
    title_location.push_str(".md");
    file_location.push_str(&title_location);
    let mut note_meta = NoteMeta::from(data);
    let now = Local::now().format(DT_FORMAT).to_string();
    // In the case that we're creating a new file
    if !PathBuf::from(&file_location).exists() {
        note_meta.metadata.insert("created".into(), now.clone());
        note_meta.metadata.insert("id".into(), now);
        let note: String = note_meta.into();
        return match fs::write(file_location, note).await {
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
        let new_location = file_location.replace(&data.old_title, &data.title);
        // Rename the file to the new title
        match fs::rename(&file_location, &new_location).await {
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
        match fs::write(file_location, final_note).await {
            Ok(()) => Ok(()),
            Err(e) => Err(WriteWikiError::WriteError(e)),
        }
    }
}

pub async fn delete(wiki_location: &str, requested_file: &str) -> Result<(), io::Error> {
    let mut file_location = String::from(wiki_location);
    if let Ok(mut file) = decode(requested_file) {
        file.to_mut().push_str(".md");
        file_location.push_str(&file);
        let file_path = PathBuf::from(file_location);
        if !file_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Could not find requested file",
            ));
        }
        fs::remove_file(file_path).await?;
        Ok(())
    } else {
        Err(std::io::Error::new(
            io::ErrorKind::InvalidInput,
            "Could not decode file name",
        ))
    }
}

pub async fn read(
    wiki_location: &str,
    requested_file: String,
    backlinks: GlobalBacklinks,
) -> Result<String, ReadPageError> {
    let file_path = get_file_path(wiki_location, &requested_file)?;
    if let Ok(note) = path_to_data_structure(&file_path).await {
        let templatted = to_template(&note);
        let link_vals = backlinks.lock().await;
        let links = link_vals.get(&templatted.page.title);
        let output = WikiPage::new(&templatted.page, links).render().await;
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

pub async fn write_index_page(user: String) {
    let ctx = IndexPage { user };
    fs::write("public/index.html", ctx.render().await)
        .await
        .unwrap();
}

pub async fn read_note_cache() -> String {
    let project_dir = ProjectDirs::from("", "", "tendril").unwrap();
    let mut data_dir = project_dir.data_dir().to_owned();
    data_dir.push("note_cache");
    fs::read_to_string(&data_dir).await.unwrap()
}

pub async fn write_note_cache(cache: String) {
    let project_dir = ProjectDirs::from("", "", "tendril").unwrap();
    let mut data_dir = project_dir.data_dir().to_owned();
    data_dir.push("note_cache");
    fs::write(data_dir, cache).await.unwrap();
}

pub fn parse_location(location: &str) -> PathBuf {
    let mut loc: String;
    if location.contains('~') {
        if let Some(dirs) = UserDirs::new() {
            let home_dir: String = dirs.home_dir().to_string_lossy().into();
            loc = location.replace('~', &home_dir);
        } else {
            loc = location.replace('~', &std::env::var("HOME").unwrap());
        }
    } else {
        loc = location.to_owned();
    }
    if !loc.ends_with(MAIN_SEPARATOR) {
        loc.push(MAIN_SEPARATOR)
    }
    PathBuf::from(loc)
}

pub fn normalize_wiki_location(wiki_location: &str) -> String {
    let location = parse_location(wiki_location);
    // Stop the process if the wiki location doesn't exist
    if !PathBuf::from(&location).exists() {
        panic!("Could not find directory at location: {:?}", location);
    }
    location.to_string_lossy().into()
}

#[cfg(test)]
mod tests {
    use super::*;
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

// TODO: this is really dependent on file system ops, won't be good if we change the storage
// backend.
pub async fn path_to_string<P: AsRef<Path> + ?Sized>(path: &P) -> Result<String, std::io::Error> {
    read_to_string(&path).await
}

pub async fn path_to_data_structure(
    path: &Path,
) -> Result<NoteMeta, Box<dyn std::error::Error + Send + Sync>> {
    let reader = path_to_string(path).await?;
    let meta = parse_meta(reader.lines(), path.to_str().unwrap());
    Ok(meta)
}

pub async fn create_journal_entry(
    location: &str,
    entry: String,
) -> Result<PatchData, std::io::Error> {
    let now = Local::now();
    let daily_file = now.format("%Y-%m-%d").to_string();
    if let Ok(exists) = get_file_path(location, &daily_file) {
        let mut entry_file = fs::read_to_string(exists.clone()).await.unwrap();
        entry_file.push_str(&format!("\n\n[{}] {}", now.format("%H:%M"), entry));
        println!("\x1b[38;5;47mdaily journal updated\x1b[0m");
        fs::write(exists, &entry_file).await?;
        Ok(NoteMeta::from(entry_file).into())
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
        fs::write(format!("{}{}.md", location, daily_file), docstring).await?;
        Ok(NoteMeta::from(daily_file).into())
    }
}

pub async fn write_archive(compressed: Vec<u8>, title: &str) {
    let project_dir = ProjectDirs::from("", "", "tendril").unwrap();
    let mut dir_path = project_dir.data_dir().to_owned();
    dir_path.push("archive");
    dir_path.push(title);
    fs::write(dir_path, compressed).await.unwrap();
}
