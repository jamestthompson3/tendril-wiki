use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use markdown::parsers::{GlobalBacklinks, TagMapping};

pub struct RefBuilder {
    pub tag_map: TagMapping,
    pub backlinks: GlobalBacklinks,
}

impl RefBuilder {
    pub fn new() -> Self {
        RefBuilder {
            tag_map: Arc::new(Mutex::new(HashMap::new())),
            backlinks: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    pub fn build(path: &PathBuf) {
        let fd = File::open(&path).unwrap();
        let reader = BufReader::new(fd);
    }
}
