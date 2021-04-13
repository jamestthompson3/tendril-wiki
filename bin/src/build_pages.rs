use markdown::parsers::{
    as_data_structure, render_template, template, template_tag_pages, update_tag_map, TagMapping,
};
use threadpool::ThreadPool;

use std::{
    collections::HashMap,
    fs::{self, read_dir},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Instant,
};

// TODO: Build out backlinks HashMap.
// the fs::write call inside of `process_file` should be extracted out so that it occurs only after
//  we've created a backlinks graph. Then, `BasicPage` can be ammended so that it includes
//  backlinks.

pub fn build(wiki_location: &String) {
    let entrypoint = PathBuf::from(shellexpand::tilde(wiki_location).to_string());
    if !Path::new("./public").exists() {
        fs::create_dir_all("./public/tags").unwrap();
    }
    let now = Instant::now();
    let tag_map = Arc::new(Mutex::new(HashMap::new()));
    let map = Arc::clone(&tag_map);
    parse_entries(entrypoint, tag_map);
    println!("compiling pages took: {}ms", now.elapsed().as_millis());
    template_tag_pages(map);
    // println!("{:?}", map.lock().unwrap());
}

fn process_file(path: PathBuf, tag_map: TagMapping) {
    let note = as_data_structure(&path);
    let templatted = template(&note);
    update_tag_map(&templatted.title, &templatted.tags, tag_map);
    let output = render_template(&templatted);
    let filename = path.file_stem().unwrap();
    fs::write(
        format!(
            "public/{}.html",
            &filename.to_str().unwrap().replace(' ', "_")
        ),
        output,
    )
    .unwrap();
}

fn parse_entries(entrypoint: PathBuf, tag_map: TagMapping) {
    let pool = ThreadPool::new(4);
    for entry in read_dir(entrypoint).unwrap() {
        let map = Arc::clone(&tag_map);
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file() {
            pool.execute(move || {
                process_file(entry.path(), map);
            });
        } else if entry.file_type().unwrap().is_dir() {
            parse_entries(entry.path(), map);
        }
    }
    pool.join();
}
