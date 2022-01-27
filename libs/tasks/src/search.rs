use std::{collections::HashMap, fs::read_dir};

use crate::{normalize_wiki_location, path_to_reader};
use futures::{stream, StreamExt};
use tokio::task;

pub type SearchResult = HashMap<String, Vec<String>>;

// Search for text within wiki pages
pub async fn context_search(term: &str, wiki_location: &str) -> Result<Vec<SearchResult>, String> {
    let location = normalize_wiki_location(wiki_location);
    let entries = read_dir(location).unwrap();
    let pipeline = stream::iter(entries)
        .filter_map(|entry| async move {
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
                    for mut line in lines.unwrap() {
                        let matches = line.to_lowercase().find(&term);
                        if matches.is_some() {
                            line = line.replace(&term, &format!("<mark>{}</mark>", term));
                            if let Some(entry) = result.get_mut(name) {
                                entry.push(line);
                            } else {
                                result.insert(name.to_string(), vec![line]);
                            }
                        }
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
        })
        .collect::<Vec<SearchResult>>()
        .await;

    Ok(pipeline)
}
