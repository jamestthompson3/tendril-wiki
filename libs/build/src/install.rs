use std::fs;

use crate::{get_data_dir_location, write_config_interactive};

pub fn install() {
    let data_dir = get_data_dir_location();
    let static_dir = data_dir.join("static");
    fs::create_dir_all(&static_dir).unwrap();
    for entry in fs::read_dir("./static").unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        fs::copy(&path, &static_dir.join(&entry.file_name())).unwrap();
    }
    write_config_interactive();
}
