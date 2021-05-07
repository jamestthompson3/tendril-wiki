use markdown::parsers::{
    parse_wiki_entry, path_to_data_structure, write_backlinks, write_entries, write_index_page,
    write_tag_index, write_tag_pages, GlobalBacklinks, ParsedPages, TagMapping,
};
use markdown::processors::{
    to_template, update_backlinks, update_tag_map, update_templatted_pages,
};
use threadpool::ThreadPool;

use std::{
    collections::HashMap,
    fs::{self, read_dir},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use crate::read_config;

/// ## TODO:
/// figure out how to encapsulate parse_entries and process_file better
/// ## NOTE:
/// Some gotchas to think about -> We're essentially keeping the whole wiki text in memory
/// which means that for very large wikis it can be a memory hog.
/// For the current size I test with ( a little over 600 pages ), it currently consumes 12MB of memory.
/// Not a huge issue, since we don't keep this in memory for serving pages, but would be nice to
/// get this down.
pub struct Builder {
    pub backlinks: GlobalBacklinks,
    pub pages: ParsedPages,
    pub tag_map: TagMapping,
}

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
        write_entries(&pages, &self.backlinks);
        write_tag_pages(map);
        write_tag_index(Arc::clone(&self.tag_map));
        write_backlinks(links);
        write_index_page(read_config().general.user);
        fs::create_dir("public/static").unwrap();
        fs::copy("./static/style.css", "./public/static/style.css").unwrap();
    }
    #[inline]
    pub fn sweep(&self, wiki_location: &str) {
        let entrypoint = parse_wiki_entry(wiki_location);
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

impl Default for Builder {
    fn default() -> Self {
        Builder::new()
    }
}

fn process_file(path: PathBuf, tags: TagMapping, backlinks: GlobalBacklinks, pages: ParsedPages) {
    let note = path_to_data_structure(&path).unwrap();
    let templatted = to_template(&note);
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
        } else if entry.file_type().unwrap().is_dir()
            && !entry.path().to_str().unwrap().contains(".git")
        {
            parse_entries(entry.path(), tags, links, pages);
        }
    }
    pool.join();
}
