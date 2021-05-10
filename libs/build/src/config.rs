use std::{env::var, fs, path::{PathBuf}};

use directories::ProjectDirs;
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Sync {
    pub use_git: bool,
    pub sync_interval: u8,
    pub branch: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct General {
    pub wiki_location: String,
    pub port: u16,
    pub user: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    pub general: General,
    pub sync: Sync,
}

pub fn get_config_location() -> (PathBuf, PathBuf) {
    let project_dir = ProjectDirs::from("", "", "tendril").unwrap();
    let config_dir = project_dir.config_dir();
    let mut config_path = PathBuf::from(config_dir);
    config_path.push("config.toml");
    (config_dir.to_owned(), config_path)
}

pub fn write_config() {
    let (dir, file) = get_config_location();
    if !file.exists() {
        fs::create_dir_all(dir).unwrap();
        let mut default_conf: Config =
            toml::from_str(&fs::read_to_string("./config/config.toml").unwrap()).unwrap();
        default_conf.general.user = get_user();
        fs::write(&file, toml::to_string(&default_conf).unwrap()).unwrap();
    }
}

pub fn read_config() -> Config {
    let (_, file) = get_config_location();
    let config: Config = toml::from_str(&fs::read_to_string(file).unwrap()).unwrap();
    config
}

fn get_user() -> String {
    match var("NAME") {
        Ok(user) => user,
        Err(_) => match var("USER") {
            Ok(user) => user,
            Err(_) => match var("USERNAME") {
                Ok(user) => user,
                Err(_) => String::from("user"),
            },
        },
    }
}
