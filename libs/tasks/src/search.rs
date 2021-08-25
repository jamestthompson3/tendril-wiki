use std::{
    fs::read_dir,
    sync::{Arc, Mutex},
};

use crate::{normalize_wiki_location, path_to_reader};
use threadpool::ThreadPool;

pub struct SearchResult {
    pub location: String,
    pub matched_text: String,
}

// length of <mark> string
const TAG_LENGTH: usize = 6;

// Search for text within wiki pages
pub fn context_search(term: &str, wiki_location: &str) -> Result<Vec<SearchResult>, String> {
    let results = Arc::new(Mutex::new(Vec::new()));
    let location = normalize_wiki_location(wiki_location);
    let pool = ThreadPool::new(4);
    for entry in read_dir(location).unwrap() {
        let entry = entry.unwrap();
        let file_name = entry.file_name();
        let name = file_name.to_str().unwrap();
        if entry.file_type().unwrap().is_file() && name.ends_with(".md") {
            let term = term.to_owned();
            let results = results.clone();
            let name = String::from(name);
            pool.execute(move || {
                let lines = path_to_reader::<_>(&entry.path());
                for mut line in lines.unwrap() {
                    if let Some(idx) = line.find(&term) {
                        line.insert_str(idx, "<mark>");
                        line.insert_str(idx + TAG_LENGTH + term.len(), "</mark>");
                        let result = SearchResult {
                            location: String::from(name.strip_suffix(".md").unwrap()),
                            matched_text: line,
                        };
                        results.lock().unwrap().push(result);
                    }
                }
            });
        }
    }
    pool.join();
    let lock = Arc::try_unwrap(results);
    if let Ok(lock) = lock {
        let final_results = lock.into_inner().expect("Mutex cannot be locked");
        Ok(final_results)
    } else {
        Ok(vec![])
    }
}

// Search for wiki by title
pub fn search(term: &str, wiki_location: &str) -> Vec<String> {
    let mut found_files = Vec::new();
    let location = normalize_wiki_location(wiki_location);
    let term = term.to_lowercase();
    for entry in read_dir(location).unwrap() {
        let entry = entry.unwrap();
        let file_name = entry.file_name();
        let name = file_name.to_str().unwrap();
        let lower_cased = name.to_lowercase();
        if entry.file_type().unwrap().is_file()
            && name.ends_with(".md")
            && lower_cased.contains(&term)
        {
            let ext_removed = name.strip_suffix(".md").unwrap();
            found_files.push(ext_removed.to_owned());
        }
    }
    found_files
}
