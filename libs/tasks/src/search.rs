use std::fs::read_dir;

use grep::{
    regex::RegexMatcherBuilder,
    searcher::{sinks::UTF8, SearcherBuilder},
};

use crate::normalize_wiki_location;

pub struct SearchResult {
    pub location: String,
    pub matched_text: String,
}

// Search for text within wiki pages
pub fn context_search(term: &str, wiki_location: &str) -> Vec<SearchResult> {
    let mut results = Vec::new();
    let location = normalize_wiki_location(&wiki_location);
    let mut searcher = SearcherBuilder::new().multi_line(true).build();
    let matcher = RegexMatcherBuilder::new()
        .line_terminator(Some(b'\n'))
        .case_smart(true)
        .build(term)
        .unwrap();
    for entry in read_dir(location).unwrap() {
        let entry = entry.unwrap();
        let file_name = entry.file_name();
        let name = file_name.to_str().unwrap();
        if entry.file_type().unwrap().is_file() && name.ends_with(".md") {
            searcher
                .search_path(
                    &matcher,
                    entry.path(),
                    UTF8(|_, matches| {
                        let result = SearchResult {
                            location: name.strip_suffix(".md").unwrap().to_owned(),
                            matched_text: matches.to_owned(),
                        };
                        results.push(result);
                        Ok(true)
                    }),
                )
                .unwrap();
        };
    }
    results
}

// Search for wiki by title
pub fn search(term: &str, wiki_location: &str) -> Vec<String> {
    let mut found_files = Vec::new();
    let location = normalize_wiki_location(&wiki_location);
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
