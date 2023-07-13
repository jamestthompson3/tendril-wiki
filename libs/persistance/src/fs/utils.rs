use directories::{ProjectDirs, UserDirs};
use std::path::{PathBuf, MAIN_SEPARATOR};

use super::{ReadPageError, WIKI_LOCATION};

pub fn get_data_dir_location() -> PathBuf {
    let project_dir = ProjectDirs::from("", "", "tendril").unwrap();
    let data_dir = project_dir.data_dir();
    data_dir.to_owned()
}

pub fn get_search_index_location() -> PathBuf {
    let data_location = get_data_dir_location();
    data_location.join("search-index")
}
pub fn get_search_file_index_location() -> PathBuf {
    let data_location = get_data_dir_location();
    data_location.join("search-index").join("file_index")
}

pub fn get_config_location() -> (PathBuf, PathBuf) {
    let project_dir = ProjectDirs::from("", "", "tendril").unwrap();
    let config_dir = project_dir.config_dir();
    let mut config_path = PathBuf::from(config_dir);
    config_path.push("config.toml");
    (config_dir.to_owned(), config_path)
}

pub fn get_wiki_location() -> PathBuf {
    WIKI_LOCATION.clone()
}

/// Returns the PathBuf if an entry exists, returns an error if the file isn't found or it couldn't
/// parse the location.
pub fn get_file_path(requested_file: &str) -> Result<PathBuf, ReadPageError> {
    let mut file_path = WIKI_LOCATION.clone();
    file_path.push(requested_file);
    file_path.set_extension("txt");

    Ok(file_path)
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

pub fn archive_file_exists(title: &str) -> bool {
    let location = get_archive_file_path(title);
    location.exists()
}

pub fn get_archive_location() -> PathBuf {
    let stored_location = get_data_dir_location();
    stored_location.join("archive")
}

pub fn get_archive_file_path(title: &str) -> PathBuf {
    let mut dir_path = get_archive_location();
    dir_path.push(title);
    dir_path
}

pub fn get_todo_location() -> PathBuf {
    let mut base_path = get_data_dir_location();
    base_path.push("todo.txt");
    base_path
}
