use std::{
    collections::BTreeMap,
    fs::read_dir,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use markdown::{
    parsers::{path_to_data_structure, GlobalBacklinks, TagMapping},
    processors::{to_template, update_backlinks, update_tag_map},
};
use threadpool::ThreadPool;

#[derive(Clone, Debug)]
pub struct RefBuilder {
    pub tag_map: TagMapping,
    pub backlinks: GlobalBacklinks,
}

impl RefBuilder {
    pub fn new() -> Self {
        RefBuilder {
            tag_map: Arc::new(Mutex::new(BTreeMap::new())),
            backlinks: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }
    pub fn build(&mut self, path: &str) {
        self.tag_map.lock().unwrap().clear();
        self.backlinks.lock().unwrap().clear();
        let map = Arc::clone(&self.tag_map);
        let links = Arc::clone(&self.backlinks);
        parse_entries(PathBuf::from(path), map, links);
    }
    pub fn tags(&self) -> TagMapping {
        Arc::clone(&self.tag_map)
    }
    pub fn links(&self) -> GlobalBacklinks {
        Arc::clone(&self.backlinks)
    }
}

impl Default for RefBuilder {
    fn default() -> Self {
        RefBuilder::new()
    }
}

// TODO: Reduce these duplicated functions, think of a better abstraction
fn parse_entries(entrypoint: PathBuf, tag_map: TagMapping, backlinks: GlobalBacklinks) {
    let pool = ThreadPool::new(num_cpus::get());
    for entry in read_dir(entrypoint).unwrap() {
        let tags = Arc::clone(&tag_map);
        let links = Arc::clone(&backlinks);
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file()
            && entry.file_name().to_str().unwrap().ends_with(".md")
        {
            pool.execute(move || {
                process_file(entry.path(), tags, links);
            });
        } else if entry.file_type().unwrap().is_dir()
            && !entry.path().to_str().unwrap().contains(".git")
        {
            parse_entries(entry.path(), tags, links);
        }
    }
    pool.join();
}

fn process_file(path: PathBuf, tags: TagMapping, backlinks: GlobalBacklinks) {
    let note = path_to_data_structure(&path).unwrap();
    let templatted = to_template(&note);
    update_tag_map(&templatted.page.title, &templatted.page.tags, tags);
    update_backlinks(&templatted.page.title, &templatted.outlinks, backlinks);
}
