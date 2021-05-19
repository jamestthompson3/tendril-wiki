use std::fs;

use directories::ProjectDirs;

use crate::write_config_interactive;

pub fn install() {
    let project_dir = ProjectDirs::from("", "", "tendril").unwrap();
    let data_dir = project_dir.data_dir();
    let static_dir = data_dir.join("static");
    fs::create_dir_all(&static_dir).unwrap();
    for entry in fs::read_dir("./static").unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        fs::copy(&path, &static_dir.join(&entry.file_name())).unwrap();
    }
    write_config_interactive();
}
