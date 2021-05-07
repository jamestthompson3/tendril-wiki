use std::{fs::read_dir, time::Instant};

use grep::{
    regex::RegexMatcher,
    searcher::{sinks::UTF8, SearcherBuilder},
};

use crate::normalize_wiki_location;

// Search for text within wiki pages
pub fn context_search(term: &str, wiki_location: &str) {
    let location = normalize_wiki_location(&wiki_location);
    let now = Instant::now();
    let mut searcher = SearcherBuilder::new().multi_line(true).build();
    let matcher = RegexMatcher::new(&term).unwrap();
    for entry in read_dir(location).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file()
            && entry.file_name().to_str().unwrap().ends_with(".md")
        {
            searcher
                .search_path(
                    &matcher,
                    entry.path(),
                    UTF8(|_line_no, _matches| {
                        // println!("line: {}, captured: {}", line_no, matches);
                        Ok(true)
                    }),
                )
                .unwrap();
        };
    }
    println!(
        "searching through wiki took: {}ms",
        now.elapsed().as_millis()
    );
}

// Search for wiki by title
pub fn search(term: &str, wiki_location: &str) -> Vec<String> {
    let mut found_files = Vec::new();
    let location = normalize_wiki_location(&wiki_location);
    for entry in read_dir(location).unwrap() {
        let entry = entry.unwrap();
        let file_name = entry.file_name();
        let name = file_name.to_str().unwrap();
        if entry.file_type().unwrap().is_file() && name.ends_with(".md") && name.contains(term) {
            let ext_removed = name.strip_suffix(".md").unwrap();
            found_files.push(ext_removed.to_owned());
        }
    }
    found_files
}
