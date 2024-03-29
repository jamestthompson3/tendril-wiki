use std::fs;

use serde_derive::{Deserialize, Serialize};

use super::utils::get_config_location;

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
    pub pass: String,
    pub version: String,
    pub media_location: String,
    pub host: String,
    pub check_for_updates: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    pub general: General,
    pub sync: Sync,
    pub externals: Option<Externals>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Externals {
    pub data: Vec<String>,
}

pub fn read_config() -> Config {
    let (_, file) = get_config_location();
    let config: Config = toml::from_str(&fs::read_to_string(file).unwrap()).unwrap();
    config
}
