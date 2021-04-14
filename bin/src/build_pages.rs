use markdown::parsers::{
    as_data_structure, render_template, template, template_backlinks, template_tag_pages,
    update_backlinks, update_tag_map, GlobalBacklinks, TagMapping,
};
use threadpool::ThreadPool;

use std::{
    collections::HashMap,
    fs::{self, read_dir},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Instant,
};

pub fn build(wiki_location: &String) {
    let entrypoint = PathBuf::from(shellexpand::tilde(wiki_location).to_string());
    if !Path::new("./public").exists() {
        fs::create_dir_all("./public/tags").unwrap();
        fs::create_dir_all("./public/links").unwrap();
    }
    let now = Instant::now();
    let tag_map = Arc::new(Mutex::new(HashMap::new()));
    let backlinks = Arc::new(Mutex::new(HashMap::new()));
    let map = Arc::clone(&tag_map);
    let links = Arc::clone(&backlinks);
    parse_entries(entrypoint, tag_map, backlinks);
    template_tag_pages(map);
    template_backlinks(links);
    println!("compiling all pages took: {}ms", now.elapsed().as_millis());
}

fn process_file(path: PathBuf, tag_map: TagMapping, backlinks: GlobalBacklinks) {
    let note = as_data_structure(&path);
    let templatted = template(&note);
    update_tag_map(&templatted.page.title, &templatted.page.tags, tag_map);
    update_backlinks(&templatted.page.title, &templatted.outlinks, backlinks);
    let output = render_template(&templatted.page);
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

fn parse_entries(entrypoint: PathBuf, tag_map: TagMapping, backlinks: GlobalBacklinks) {
    let pool = ThreadPool::new(num_cpus::get());
    for entry in read_dir(entrypoint).unwrap() {
        let tags = Arc::clone(&tag_map);
        let links = Arc::clone(&backlinks);
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file() {
            pool.execute(move || {
                process_file(entry.path(), tags, links);
            });
        } else if entry.file_type().unwrap().is_dir() {
            parse_entries(entry.path(), tags, links);
        }
    }
    pool.join();
}
