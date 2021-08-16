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
    let location = normalize_wiki_location(wiki_location);
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
                        let match_str = matches.to_string();
                        let match_vec: Vec<&str> = match_str.split(' ').collect();
                        let matched_idx = match_vec
                            .iter()
                            .position(|i| i.to_lowercase().contains(&term.to_lowercase()));
                        if let Some(idx) = matched_idx {
                            let result = highlight_term(match_vec, idx, &term.to_lowercase(), name);
                            results.push(result);
                        } else {
                            let result = SearchResult {
                                location: name.strip_suffix(".md").unwrap().to_owned(),
                                matched_text: matches.to_owned(),
                            };
                            results.push(result);
                        }
                        Ok(true)
                    }),
                )
                .unwrap();
        };
    }
    results
}

fn highlight_term(mut match_vec: Vec<&str>, idx: usize, term: &str, name: &str) -> SearchResult {
    let location = name.strip_suffix(".md").unwrap().to_owned();
    let matched_string = match_vec[idx];
    if matched_string.to_lowercase() != term {
        // frustration
        let string_parts: Vec<&str> = matched_string.split(term).collect();
        // ["f", "ration"]
        let recombinated_string = string_parts.join(&format!("<mark>{}</mark>", term));
        // bleh. rethink later
        match_vec.remove(idx);
        let mut stringified = match_vec;
        stringified.insert(idx, &recombinated_string);
        let matched_text = stringified.join(" ");
        SearchResult {
            location,
            matched_text,
        }
    } else {
        match_vec.insert(idx, "<mark>");
        match_vec.insert(idx + 2, "</mark>");
        let matched_text = match_vec.join(" ");
        SearchResult {
            location,
            matched_text,
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlights_exact_term() {
        let matches = vec!["rust", "is", "a", "compiled", "programming", "language"];
        let idx = 0;
        let term = "rust";
        let name = "learn_rust.md";
        let result = highlight_term(matches, idx, term, name);
        let expected = SearchResult {
            location: String::from("learn_rust"),
            matched_text: String::from("<mark> rust </mark> is a compiled programming language"),
        };
        assert_eq!(result.location, expected.location);
        assert_eq!(result.matched_text, expected.matched_text);
    }

    #[test]
    fn highlights_context_term() {
        let matches = vec!["trust", "is", "important", "when", "building", "a", "team"];
        let idx = 0;
        let term = "rust";
        let name = "team_building.md";
        let result = highlight_term(matches, idx, term, name);
        let expected = SearchResult {
            location: String::from("team_building"),
            matched_text: String::from("t<mark>rust</mark> is important when building a team"),
        };
        assert_eq!(result.location, expected.location);
        assert_eq!(result.matched_text, expected.matched_text);
    }
}
