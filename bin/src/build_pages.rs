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

#[derive(Debug)]
pub struct Builder {
    pub backlinks: GlobalBacklinks,
    pub pages: ParsedPages,
    pub tag_map: TagMapping,
}

/// TODO: figure out how to encapsulate parse_entries and process_file better

impl Builder {
    pub fn new() -> Self {
        Builder {
            tag_map: Arc::new(Mutex::new(HashMap::new())),
            backlinks: Arc::new(Mutex::new(HashMap::new())),
            pages: Arc::new(Mutex::new(Vec::new())),
        }
    }
    pub fn compile_all(&self) {
        let map = Arc::clone(&self.tag_map);
        let links = Arc::clone(&self.backlinks);
        let pages = Arc::clone(&self.pages);
        let now = Instant::now();
        template_entries(&pages, &self.backlinks);
        template_tag_pages(map);
        template_backlinks(links);
        println!("compiling all pages took: {}ms", now.elapsed().as_millis());
    }
    pub fn sweep(&self, wiki_location: &String) {
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
        let map = Arc::clone(&self.tag_map);
        let links = Arc::clone(&self.backlinks);
        let pages = Arc::clone(&self.pages);
        parse_entries(entrypoint, map, links, pages);
    }
}

fn process_file(path: PathBuf, tags: TagMapping, backlinks: GlobalBacklinks, pages: ParsedPages) {
    let note = as_data_structure(&path);
    let templatted = template(&note);
    update_tag_map(&templatted.page.title, &templatted.page.tags, tags);
    update_backlinks(&templatted.page.title, &templatted.outlinks, backlinks);
    update_templatted_pages(templatted.page, pages);
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
