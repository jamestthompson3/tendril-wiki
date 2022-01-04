use std::fs;

use crate::{get_data_dir_location, write_config_interactive};

fn prep_files() {
    let data_dir = get_data_dir_location();
    let static_dir = data_dir.join("static");
    let template_dir = data_dir.join("templates");
    let cache_file = data_dir.join("note_cache");
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
        fs::copy(&path, &static_dir.join(&entry.file_name())).unwrap();
    }
    for entry in fs::read_dir("./templates").unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        fs::copy(&path, &template_dir.join(&entry.file_name())).unwrap();
    }
}

pub fn install() {
    prep_files();
    write_config_interactive();
}

pub fn update() {
    prep_files();
    println!("Files updated!");

}
