use std::{
    fs,
    path::{Path, PathBuf},
    process::exit,
};

use persistance::fs::{
    config::Config,
    utils::{get_config_location, get_data_dir_location, get_wiki_location},
};
use tasks::hash_password;
use wikitext::parsers::Note;

use crate::{gen_config_interactive, ConfigOptions};

fn prep_files() {
    let data_dir = get_data_dir_location();
    let static_dir = data_dir.join("static");
    let template_dir = data_dir.join("templates");
    let archive_dir = data_dir.join("archive");
    let cache_file = data_dir.join("note_cache");
    fs::create_dir_all(&archive_dir).unwrap();
    fs::create_dir_all(&static_dir).unwrap();
    fs::create_dir_all(&template_dir).unwrap();
    if !cache_file.exists() {
        fs::File::create(cache_file).unwrap();
    }
    let version = env!("CARGO_PKG_VERSION");
    for entry in fs::read_dir("./static").unwrap() {
        let entry = entry.unwrap();
        // update our service worker version
        // TODO: Don't change the master version of the sw.js file.
        if entry.file_name() == "sw.js" {
            let mut f = fs::read_to_string(&entry.path()).unwrap();
            f = f.replace("%VERSION%", version);
            fs::write(entry.path(), f).unwrap();
        }
        let path = entry.path();
        if entry.metadata().unwrap().is_dir() {
            let dir_loc = static_dir.join(&entry.file_name());
            fs::create_dir_all(&dir_loc).unwrap();
            for entry in fs::read_dir(&dir_loc).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                fs::copy(&path, &dir_loc.join(&entry.file_name())).unwrap();
            }
        } else {
            fs::copy(&path, &static_dir.join(&entry.file_name())).unwrap();
        }
    }
    for entry in fs::read_dir("./templates").unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        fs::copy(&path, &template_dir.join(&entry.file_name())).unwrap();
    }
}

pub fn install() {
    prep_files();
    let options = gen_config_interactive();
    bootstrap_initial_files(options);
}

pub fn update() {
    prep_files();
    println!("Files updated!");
}

pub fn migrate() {
    migrate_md_to_wikitext();
}

fn migrate_md_to_wikitext() {
    let mut backup_dir = get_wiki_location();
    let original_dir = get_wiki_location();
    backup_dir.pop();
    backup_dir.push("tendril-backup");
    fs::create_dir_all(&backup_dir).unwrap();
    process_migration_dir(original_dir, backup_dir);
}

fn process_migration_dir(original_dir: PathBuf, backup_dir: PathBuf) {
    let wiki_dir = get_wiki_location();
    let data_dir = get_data_dir_location();
    for file in fs::read_dir(original_dir).unwrap() {
        let entry = file.unwrap();
        if entry.path().to_str().unwrap().contains(".git") {
            continue;
        }
        if entry.metadata().unwrap().is_dir() {
            process_migration_dir(entry.path(), backup_dir.clone());
            for file in fs::read_dir(entry.path()).unwrap() {
                let entry = file.unwrap();
                fs::rename(entry.path(), wiki_dir.join(entry.file_name())).unwrap();
            }
            fs::remove_dir(entry.path()).unwrap();
        } else {
            let target = backup_dir.join(entry.file_name());
            let entry_path = entry.path();
            process_migration_file(entry_path, target, &data_dir);
        }
    }
}

fn process_migration_file(entry_path: PathBuf, target: PathBuf, data_dir: &Path) {
    fs::copy(&entry_path, target).unwrap();
    let file_name = entry_path.file_name().unwrap();
    let file_name = file_name.to_str().unwrap();
    if file_name == "todo.txt" {
        let new_loc = data_dir.join(file_name);
        fs::rename(&entry_path, &new_loc).unwrap();
        return;
    }
    if file_name.ends_with("md") {
        let contents = fs::read_to_string(&entry_path).unwrap();

        let replaced_lines = contents
            .lines()
            .enumerate()
            .filter_map(|(idx, line)| {
                if idx == 0 && (line == "---" || line.is_empty()) {
                    return None;
                }
                if line == "---" {
                    // case where the original file started with an empty line or malformed
                    // metadata
                    if idx == 1 {
                        return None;
                    }
                    return Some("\n".into());
                }
                if line.starts_with('#') {
                    let cleaned_line = line.replace('#', "");
                    let formatted = format!("# {}", cleaned_line);
                    return Some(formatted);
                }
                Some(line.into())
            })
            .collect::<Vec<String>>()
            .join("\n");
        let mut note: Note = replaced_lines.into();
        if let Some(tags) = note.header.get("tags") {
            if tags.contains("bookmark") {
                if let Some(content_type) = note.header.get("content-type") {
                    if content_type != "html" {
                        note.header.insert("content-type".into(), "html".into());
                    }
                } else {
                    note.header.insert("content-type".into(), "html".into());
                }
            }
        }
        let mut path = entry_path.clone();
        path.set_extension("txt");
        fs::write(path, std::convert::Into::<String>::into(note)).unwrap();
        fs::remove_file(&entry_path).unwrap();
    }
}

fn bootstrap_initial_files(options: ConfigOptions) {
    let (parsed_location, parsed_media_location, enable_sync, branch, user, password) = options;
    let (mut dir, file) = get_config_location();
    if !file.exists() {
        fs::create_dir_all(&dir).unwrap();
        let mut default_conf: Config =
            toml::from_str(&fs::read_to_string("config/config.toml").unwrap()).unwrap();
        default_conf.general.user = user;
        default_conf.general.wiki_location = parsed_location.to_string_lossy().into();
        default_conf.general.media_location = parsed_media_location.to_string_lossy().into();
        // Create the wiki and media paths if they don't already exist
        fs::create_dir_all(parsed_location).unwrap();
        if !parsed_media_location.exists() {
            fs::create_dir_all(parsed_media_location).unwrap();
        }
        default_conf.sync.use_git = enable_sync;
        default_conf.sync.branch = branch.unwrap_or_else(|| "".to_string());
        if let Some(password) = password {
            let pass = hash_password(password.as_bytes());
            default_conf.general.pass = pass;
        }
        fs::write(&file, toml::to_string(&default_conf).unwrap()).unwrap();
        dir.push("userstyles.css");
        fs::copy("./config/userstyles.css", dir).unwrap();
    } else {
        println!("\nWiki location already exists, exiting...");
        exit(0);
    }
}
