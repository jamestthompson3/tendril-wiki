use std::{fs, process::exit};

use persistance::fs::utils::{get_config_location, get_data_dir_location};
use tasks::hash_password;

use crate::{gen_config_interactive, Config, ConfigOptions};

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
