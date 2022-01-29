use std::{
    collections::HashMap,
    fs::{read_dir, DirEntry},
};

use crate::{normalize_wiki_location, path_to_reader};
use futures::{stream, StreamExt};
use tokio::task;

pub type SearchResult = HashMap<String, Vec<String>>;

// Search for text within wiki pages
pub async fn context_search(term: &str, wiki_location: &str) -> Result<Vec<SearchResult>, String> {
    let location = normalize_wiki_location(wiki_location);
    let entries = read_dir(location).unwrap();
    let pipeline = stream::iter(entries)
        .filter_map(|entry| async move { search_in_dir_entries(entry, term).await })
        .collect::<Vec<SearchResult>>()
        .await;

    Ok(pipeline)
}

async fn search_in_dir_entries(
    entry: Result<DirEntry, std::io::Error>,
    term: &str,
) -> Option<SearchResult> {
    let entry = entry.unwrap();
    let file_name = entry.file_name();
    let name = file_name.to_str().unwrap();
    if entry.file_type().unwrap().is_file() && name.ends_with(".md") {
        let term = term.to_owned();
        let name = String::from(name);
        let join = task::spawn(async move {
            let mut result: SearchResult = HashMap::new();
            let lines = path_to_reader::<_>(&entry.path());
            let name = name.strip_suffix(".md").unwrap();
            if name.to_lowercase().contains(&term.to_lowercase()) {
                result.insert(name.to_string(), vec![String::with_capacity(0)]);
            }
            for line in lines.unwrap() {
                search_lines(line, &term, &mut result, name);
            }
            result
        });
        let eventual_value = join.await.unwrap();
        if !eventual_value.is_empty() {
            return Some(eventual_value);
        }
        None
    } else {
        None
    }
}

const OPEN_TAG_LENGTH: usize = 6;
const CLOSE_TAG_LENGTH: usize = 7;

fn search_lines(mut line: String, term: &str, result: &mut SearchResult, name: &str) {
    let readline = line.clone().to_lowercase();
    let matches = readline
        .match_indices(&term)
        .collect::<Vec<(usize, &str)>>();
    if !matches.is_empty() {
        for (pointer, (idx, t)) in matches.into_iter().enumerate() {
            let current_pos = idx + (pointer * (OPEN_TAG_LENGTH + CLOSE_TAG_LENGTH));
            let closing_tag = current_pos + OPEN_TAG_LENGTH + t.len();
            line.insert_str(current_pos, "<mark>");
            line.insert_str(closing_tag, "</mark>");
        }
        if let Some(entry) = result.get_mut(name) {
            entry.push(line);
        } else {
            result.insert(name.to_string(), vec![line]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_a_single_match() {
        let line = String::from("Tests are helpful!");
        let term = "help";
        let mut result: SearchResult = HashMap::new();
        let name = "test_file";
        search_lines(line, term, &mut result, name);
        let inserted_match = result.get(name).unwrap();
        assert_eq!(inserted_match, &vec!["Tests are <mark>help</mark>ful!"]);
    }

    #[test]
    fn returns_multiple_matches() {
        let line = String::from(
            "Tests are helpful! They can help you find bugs. They can help you develop faster.",
        );
        let term = "help";
        let mut result: SearchResult = HashMap::new();
        let name = "test_file";
        search_lines(line, term, &mut result, name);
        let inserted_match = result.get(name).unwrap();
        assert_eq!(
            inserted_match,
            &vec!["Tests are <mark>help</mark>ful! They can <mark>help</mark> you find bugs. They can <mark>help</mark> you develop faster."]
        );
    }

    #[test]
    fn returns_a_single_match_case_insensitive() {
        let line = String::from("Tests are HELPful!");
        let term = "help";
        let mut result: SearchResult = HashMap::new();
        let name = "test_file";
        search_lines(line, term, &mut result, name);
        let inserted_match = result.get(name).unwrap();
        assert_eq!(inserted_match, &vec!["Tests are <mark>HELP</mark>ful!"]);
    }

    #[test]
    fn returns_multiple_matches_case_insensitive() {
        let line = String::from(
            "Tests are HeLpful! They can HELP you find bugs. They can hElp you develop faster.",
        );
        let term = "help";
        let mut result: SearchResult = HashMap::new();
        let name = "test_file";
        search_lines(line, term, &mut result, name);
        let inserted_match = result.get(name).unwrap();
        assert_eq!(
            inserted_match,
            &vec!["Tests are <mark>HeLp</mark>ful! They can <mark>HELP</mark> you find bugs. They can <mark>hElp</mark> you develop faster."]
        );
    }
}
