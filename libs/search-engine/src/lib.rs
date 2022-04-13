use build::get_data_dir_location;
use searcher::search;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, usize};

/// Heavy inspiration / code taken from: https://github.com/thesephist/monocle
use tokio::fs::{read_to_string, write};

use crate::indexer::{Notebook, Proccessor};

mod indexer;
mod searcher;
mod tokenizer;

pub(crate) type Tokens = HashMap<String, usize>;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Doc {
    id: String,
    tokens: Tokens,
    content: String,
}

pub async fn build_search_index(location: PathBuf) {
    let mut p = Notebook::default();
    println!("Indexing notes...");
    p.load(&location).await;
    let index = p.index();
    println!("Writing persistent index...");
    write_search_index(&index).await;
    println!("Writing document files...");
    write_doc_index(&p.docs_to_idx()).await;
}

pub async fn semantic_search(term: &str) -> Vec<(String, String)> {
    let index_location = get_data_dir_location();
    let results = search(term, index_location).await;
    results
        .into_iter()
        .map(|d| (d.id, d.content))
        .collect::<Vec<(String, String)>>()
}

pub(crate) async fn write_doc_index(doc_idx: &HashMap<String, &Doc>) {
    let stored_location = get_data_dir_location();
    let loc = stored_location.join("docs.json");
    write(loc, serde_json::to_string(doc_idx).unwrap())
        .await
        .unwrap();
}
pub(crate) async fn write_search_index(search_idx: &HashMap<String, Vec<String>>) {
    let stored_location = get_data_dir_location();
    let loc = stored_location.join("search-index.json");
    write(loc, serde_json::to_string(search_idx).unwrap())
        .await
        .unwrap();
}

pub(crate) async fn read_doc_index(loc: PathBuf) -> HashMap<String, Doc> {
    let doc_idx = read_to_string(&loc).await.unwrap();
    let doc_idx: HashMap<String, Doc> = serde_json::from_str(&doc_idx).unwrap();
    doc_idx
}

pub(crate) async fn read_search_index(loc: PathBuf) -> HashMap<String, Vec<String>> {
    let search_idx = read_to_string(&loc).await.unwrap();
    let search_idx: HashMap<String, Vec<String>> = serde_json::from_str(&search_idx).unwrap();
    search_idx
}

pub(crate) async fn patch_search_index(
    doc: Doc,
    mut search_idx: HashMap<String, Vec<String>>,
    mut doc_idx: HashMap<String, Doc>,
) {
    let mut removed_tokens = Vec::new();
    let mut added_tokens = Vec::new();
    // TODO: Don't clone so much
    if let Some(old_version) = doc_idx.get_mut(&doc.id) {
        let old_tokens = old_version.tokens.clone();
        let mut old_token_strings = old_tokens.keys();
        let mut new_token_strings = doc.tokens.keys();
        for token in old_token_strings.clone() {
            if !new_token_strings.any(|f| f == token) {
                removed_tokens.push(token);
            }
        }
        for token in new_token_strings.clone() {
            if !old_token_strings.any(|f| f == token) {
                added_tokens.push(token)
            }
        }

        for token in removed_tokens {
            old_version.tokens.remove(token).unwrap();
            if let Some(search_token) = search_idx.get_mut(token) {
                *search_token = search_token
                    .iter()
                    .filter(|&f| f != &doc.id)
                    .map(|t| t.to_owned())
                    .collect::<Vec<String>>();
            }
        }

        for token in added_tokens {
            let doc_id = doc.id.clone();
            if let Some(search_token) = old_version.tokens.get_mut(token) {
                *search_token += 1;
            } else {
                old_version.tokens.insert(token.clone(), 1);
            }
            if let Some(search_token) = search_idx.get_mut(token) {
                search_token.push(doc_id);
            } else {
                search_idx.insert(token.clone(), vec![doc_id]);
            }
        }
        write_search_index(&search_idx).await;
        let mut doc_map = HashMap::with_capacity(1);
        doc_map.insert(doc.id.clone(), &doc);
        write_doc_index(&doc_map).await;
    }
}
