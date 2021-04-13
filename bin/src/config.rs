use std::{fs, path::PathBuf};

use directories::ProjectDirs;
use serde_derive::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub wiki_location: String,
    pub port: u16,
}

pub fn read_config() -> Config {
    let project_dir = ProjectDirs::from("", "", "tendril").unwrap();
    let config_dir = project_dir.config_dir();
    let mut config_path = PathBuf::from(config_dir);
    config_path.push("config.toml");
    if !config_path.exists() {
        fs::create_dir_all(config_dir).unwrap();
        // FIXME: Later use includestr!
        let default_conf = format!("wiki_location = \"~/wiki\"\nport = 6683");
        fs::write(&config_path, default_conf).unwrap();
    }
    let config: Config = toml::from_str(&fs::read_to_string(config_path).unwrap()).unwrap();
    config
}
