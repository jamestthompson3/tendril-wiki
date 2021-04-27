use std::{fs, path::PathBuf, env::var};

use directories::ProjectDirs;
use serde_derive::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub wiki_location: String,
    pub port: u16,
    pub user: String
}

pub fn read_config() -> Config {
    let project_dir = ProjectDirs::from("", "", "tendril").unwrap();
    let config_dir = project_dir.config_dir();
    let mut config_path = PathBuf::from(config_dir);
    config_path.push("config.toml");
    if !config_path.exists() {
        fs::create_dir_all(config_dir).unwrap();
        // FIXME: Later use includestr!
        let default_conf = format!("wiki_location = \"~/wiki\"\nport = 6683\nuser = \"{}\"", get_user());
        fs::write(&config_path, default_conf).unwrap();
    }
    let config: Config = toml::from_str(&fs::read_to_string(config_path).unwrap()).unwrap();
    config
}

fn get_user() -> String {
        match var("NAME") {
            Ok(user) => return user,
            Err(_) => match var("USER") {
                Ok(user) => return user,
                Err(_) => match var("USERNAME") {
                    Ok(user) => return user,
                    Err(_) => return String::from("user"),
                },
            },
        }
}
