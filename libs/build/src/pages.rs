use markdown::parsers::{path_to_data_structure, ParsedPages};

use markdown::processors::{to_template, update_templatted_pages};
use persistance::fs::{write_entries, write_index_page};
use render::{write_backlinks, write_tag_index, write_tag_pages, GlobalBacklinks, TagMapping};
use tasks::CompileState;
use threadpool::ThreadPool;

use std::{
    collections::BTreeMap,
    fs::{self, read_dir},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use crate::{get_config_location, build_global_store, read_config};

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
            tag_map: Arc::new(Mutex::new(BTreeMap::new())),
            backlinks: Arc::new(Mutex::new(BTreeMap::new())),
            pages: Arc::new(Mutex::new(Vec::new())),
        }
    }
    pub fn compile_all(&self) {
        let map = Arc::clone(&self.tag_map);
        let links = Arc::clone(&self.backlinks);
        let pages = Arc::clone(&self.pages);
        write_entries(&pages, &self.backlinks, CompileState::Static);
        write_tag_pages(map, &pages, CompileState::Static);
        write_tag_index(Arc::clone(&self.tag_map), CompileState::Static);
        write_backlinks(links, CompileState::Static);
        write_index_page(read_config().general.user, CompileState::Static);
        let mut config_dir = get_config_location().0;
        config_dir.push("userstyles.css");
        fs::create_dir("public/static").unwrap();
        fs::create_dir("public/config").unwrap();
        fs::copy("./static/style.css", "./public/static/style.css").unwrap();
        fs::copy(config_dir, "./public/config/userstyles.css").unwrap();
    }

    pub fn sweep(&self, wiki_location: &str) {
        if !Path::new("./public").exists() {
            fs::create_dir_all("./public/tags").unwrap();
            fs::create_dir_all("./public/links").unwrap();
        }
        let map = Arc::clone(&self.tag_map);
        let links = Arc::clone(&self.backlinks);
        let pages = Arc::clone(&self.pages);
        parse_entries(PathBuf::from(wiki_location), map, links, pages);
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
    build_global_store(&templatted.page.title, &templatted.outlinks, backlinks, tags, &templatted.page.tags);
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
