use markdown::parsers::{
    as_data_structure, template, template_backlinks, template_entries, template_tag_pages,
    update_backlinks, update_tag_map, update_templatted_pages, GlobalBacklinks, ParsedPages,
    TagMapping,
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
    let entrypoint: PathBuf;
    if wiki_location.contains('~') {
        entrypoint = PathBuf::from(wiki_location.replace('~', &std::env::var("HOME").unwrap()));
    } else {
        entrypoint = PathBuf::from(wiki_location);
    }
    if !Path::new("./public").exists() {
        fs::create_dir_all("./public/tags").unwrap();
        fs::create_dir_all("./public/links").unwrap();
    }
    let now = Instant::now();
    let tag_map = Arc::new(Mutex::new(HashMap::new()));
    let backlinks = Arc::new(Mutex::new(HashMap::new()));
    let rendered_pages: ParsedPages = Arc::new(Mutex::new(Vec::new()));
    let map = Arc::clone(&tag_map);
    let links = Arc::clone(&backlinks);
    let pages = Arc::clone(&rendered_pages);
    parse_entries(entrypoint, tag_map, backlinks, rendered_pages);
    template_entries(&pages, &links);
    template_tag_pages(map);
    template_backlinks(links);
    println!("compiling all pages took: {}ms", now.elapsed().as_millis());
}

/// FIXME: The only reason we have to keep passing down these mutexes through these functions
/// is so that `parse_entries` can be recursive. Maybe make a flat directory structure?

fn process_file(
    path: PathBuf,
    tag_map: TagMapping,
    backlinks: GlobalBacklinks,
    rendered_pages: ParsedPages,
) {
    let note = as_data_structure(&path);
    let templatted = template(&note);
    update_tag_map(&templatted.page.title, &templatted.page.tags, tag_map);
    update_backlinks(&templatted.page.title, &templatted.outlinks, backlinks);
    update_templatted_pages(templatted.page, rendered_pages);
}

fn parse_entries(
    entrypoint: PathBuf,
    tag_map: TagMapping,
    backlinks: GlobalBacklinks,
    rendered_pages: ParsedPages,
) {
    let pool = ThreadPool::new(num_cpus::get());
    for entry in read_dir(entrypoint).unwrap() {
        let tags = Arc::clone(&tag_map);
        let links = Arc::clone(&backlinks);
        let pages = Arc::clone(&rendered_pages);
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file()
            && entry.file_name().to_str().unwrap().ends_with(".md")
        {
            pool.execute(move || {
                process_file(entry.path(), tags, links, pages);
            });
        } else if entry.file_type().unwrap().is_dir() {
            if !entry.path().to_str().unwrap().contains(".git") {
                parse_entries(entry.path(), tags, links, pages);
            }
        }
    }
    pool.join();
}
