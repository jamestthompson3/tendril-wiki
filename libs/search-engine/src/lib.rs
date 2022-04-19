use build::get_data_dir_location;
use indexer::tokenize_note_meta;
use markdown::parsers::NoteMeta;
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
type DocIdx = HashMap<String, Doc>;
type SearchIdx = HashMap<String, Vec<String>>;

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

pub(crate) async fn write_doc_index<T: Serialize>(doc_idx: T) {
    let stored_location = get_data_dir_location();
    let loc = stored_location.join("docs.json");
    write(loc, serde_json::to_string(&doc_idx).unwrap())
        .await
        .unwrap();
}

pub(crate) async fn write_search_index(search_idx: &SearchIdx) {
    let stored_location = get_data_dir_location();
    let loc = stored_location.join("search-index.json");
    write(loc, serde_json::to_string(search_idx).unwrap())
        .await
        .unwrap();
}

pub(crate) async fn read_doc_index() -> DocIdx {
    let stored_location = get_data_dir_location();
    let loc = stored_location.join("docs.json");
    let doc_idx = read_to_string(&loc).await.unwrap();
    let doc_idx: DocIdx = serde_json::from_str(&doc_idx).unwrap();
    doc_idx
}

pub(crate) async fn read_search_index() -> SearchIdx {
    let stored_location = get_data_dir_location();
    let loc = stored_location.join("search-index.json");
    let search_idx = read_to_string(&loc).await.unwrap();
    let search_idx: SearchIdx = serde_json::from_str(&search_idx).unwrap();
    search_idx
}

pub async fn patch_search_from_update(note: &NoteMeta) {
    let search_idx = read_search_index().await;
    let doc_idx = read_doc_index().await;
    let doc = tokenize_note_meta(note);
    let (search_idx, doc_idx) = patch_search_index(doc, search_idx, doc_idx).await.unwrap();
    write_search_index(&search_idx).await;
    write_doc_index(&doc_idx).await;
}

pub async fn delete_entry_from_update(entry: &str) {
    let search_idx = read_search_index().await;
    let doc_idx = read_doc_index().await;
    let (search_idx, doc_idx) = delete_entry_from_index(search_idx, doc_idx, entry).await;
    write_doc_index(doc_idx).await;
    write_search_index(&search_idx).await;
}

async fn delete_entry_from_index(
    mut search_idx: SearchIdx,
    mut doc_idx: DocIdx,
    entry: &str,
) -> (SearchIdx, DocIdx) {
    let doc = doc_idx
        .get(entry)
        .expect("Could not find doc marked for removal");
    for token in doc.tokens.keys() {
        let matched_documents = search_idx
            .get_mut(token)
            .expect("Improperly index search term.");
        *matched_documents = matched_documents
            .iter()
            .filter(|i| *i != entry)
            .map(|i| i.to_owned())
            .collect::<Vec<String>>();
        if matched_documents.is_empty() {
            search_idx.remove(token).unwrap();
        }
    }
    doc_idx.remove(entry).unwrap();
    (search_idx, doc_idx)
}

async fn patch_search_index(
    doc: Doc,
    mut search_idx: SearchIdx,
    mut doc_idx: DocIdx,
) -> Option<(SearchIdx, DocIdx)> {
    let mut removed_tokens = Vec::new();
    let mut added_tokens = Vec::new();
    // TODO: Don't clone so much
    if let Some(old_version) = doc_idx.get_mut(&doc.id) {
        let old_tokens = old_version.tokens.clone();
        for token in old_tokens.keys() {
            if !doc.tokens.keys().any(|f| f == token) {
                removed_tokens.push(token);
            }
        }
        for token in doc.tokens.keys() {
            if !old_tokens.keys().any(|f| f == token) {
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
                if search_token.is_empty() {
                    search_idx.remove(token).unwrap();
                }
            }
        }

        for token in dbg!(added_tokens) {
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
        let mut doc_map = HashMap::with_capacity(1);
        doc_map.insert(doc.id.clone(), doc);
        Some((search_idx, doc_map))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn delete_entry_successfully() {
        let mut search_idx: SearchIdx = HashMap::new();
        search_idx.insert("test".into(), vec!["test_doc".into(), "another_doc".into()]);
        search_idx.insert("token".into(), vec!["test_doc".into()]);
        let mut doc_idx: DocIdx = HashMap::new();
        let doc = Doc {
            id: "test_doc".into(),
            tokens: HashMap::from([("test".into(), 1), ("token".into(), 1)]),
            content: "test token".into(),
        };
        doc_idx.insert("test_doc".into(), doc);
        let (new_search, new_doc) = delete_entry_from_index(search_idx, doc_idx, "test_doc").await;
        assert_eq!(new_search.get("token"), None);
        assert_eq!(new_search.get("test"), Some(&vec!["another_doc".into()]));
        assert!(new_doc.is_empty());
    }
    #[tokio::test]
    async fn patches_entry_successfully() {
        let mut search_idx: SearchIdx = HashMap::new();
        search_idx.insert("test".into(), vec!["test_doc".into(), "another_doc".into()]);
        search_idx.insert("token".into(), vec!["test_doc".into()]);
        let mut doc_idx: DocIdx = HashMap::new();
        let doc = Doc {
            id: "test_doc".into(),
            tokens: HashMap::from([("test".into(), 1), ("token".into(), 1)]),
            content: "test token".into(),
        };
        doc_idx.insert("test_doc".into(), doc);

        let updated_doc = Doc {
            id: "test_doc".into(),
            tokens: HashMap::from([("cool".into(), 1), ("info".into(), 1)]),
            content: "cool info".into(),
        };
        let (new_search, _new_docs) = patch_search_index(updated_doc, search_idx, doc_idx)
            .await
            .unwrap();
        let new_search = dbg!(new_search);
        let updated_search_term_info = new_search.get("info");
        let updated_search_term_cool = new_search.get("cool");
        let search_term_test = new_search.get("test");
        let search_term_token = new_search.get("token");
        assert_eq!(updated_search_term_info, Some(&vec!["test_doc".into()]));
        assert_eq!(updated_search_term_cool, Some(&vec!["test_doc".into()]));
        assert_eq!(search_term_test, Some(&vec!["another_doc".into()]));
        assert_eq!(search_term_token, None);
    }
}
