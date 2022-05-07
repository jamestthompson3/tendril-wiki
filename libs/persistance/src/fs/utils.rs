use directories::ProjectDirs;
use std::path::PathBuf;

pub fn get_data_dir_location() -> PathBuf {
    let project_dir = ProjectDirs::from("", "", "tendril").unwrap();
    let data_dir = project_dir.data_dir();
    data_dir.to_owned()
}

pub fn get_config_location() -> (PathBuf, PathBuf) {
    let project_dir = ProjectDirs::from("", "", "tendril").unwrap();
    let config_dir = project_dir.config_dir();
    let mut config_path = PathBuf::from(config_dir);
    config_path.push("config.toml");
    (config_dir.to_owned(), config_path)
}
