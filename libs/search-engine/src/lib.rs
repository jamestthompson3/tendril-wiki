use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use indexer::notebook::Notebook;
use persistance::fs::utils::{get_archive_location, get_data_dir_location};
use searcher::{highlight_matches, search};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{create_dir, read, write},
    io::Write,
    path::PathBuf,
    process::exit,
    usize,
};
use thiserror::Error;
use tokenizer::tokenize;
use wikitext::parsers::Note;

/// Heavy inspiration / code taken from: https://github.com/thesephist/monocle
use tokio::fs::{read_to_string, remove_file};

use crate::indexer::{archive::Archive, Proccessor};

mod indexer;
mod searcher;
mod tokenizer;

pub type Tokens = HashMap<String, Vec<(String, f32)>>;
type DocIdx = HashMap<String, Doc>;
type SearchIdx = HashMap<String, Vec<String>>;

#[derive(Error, Debug)]
pub enum SearchIndexReadErr {
    #[error("Could not find file")]
    NotExistErr,
    #[error("Could not deserialize file")]
    DeserErr(bincode::Error),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct Doc {
    id: String,
    tokens: Tokens,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Indicies {
    search_idx: SearchIdx,
    doc_idx: DocIdx,
}

pub fn build_search_index(location: &str) {
    let archive_location = get_archive_location();
    let mut n = Notebook::default();
    let mut a = Archive::default();
    println!("<indexing notes>");
    n.load(&PathBuf::from(location));
    a.load(&archive_location);
    for (key, value) in a.tokens.iter() {
        if let Some(exists) = n.tokens.get_mut(key) {
            exists.extend(value.to_owned());
        } else {
            n.tokens.insert(key.to_owned(), value.to_owned());
        }
    }
    write_search_index(&n.tokens);
}

pub async fn dump_search_index() -> Result<Tokens, bincode::Error> {
    todo!("REMOVE");
    // read_search_index()
}

pub async fn semantic_search(term: &str) -> Vec<String> {
    search(term).await
}

pub(crate) fn write_search_index(search_idx: &Tokens) {
    // TODO: Handle file/dir existing already.
    let stored_location = get_data_dir_location();
    let mut loc = stored_location.join("search-index");
    create_dir(&loc).unwrap();
    for (key, value) in search_idx.iter() {
        let bytes = bincode::serialize(value).unwrap();
        let file_loc = loc.join(key);
        match write(file_loc, bytes) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("Could not write key -> {}\n{}", key, e);
                exit(1);
            }
        }
    }
}

pub(crate) fn read_search_index(filename: &str) -> Result<Vec<(String, f32)>, SearchIndexReadErr> {
    let index_location = get_data_dir_location();
    let read_loc = index_location.join("search-index").join(filename);
    match read(read_loc) {
        Ok(content) => {
            let deserialized_freqs = bincode::deserialize(&content);
            match deserialized_freqs  {
                Ok(tokens) => Ok(tokens),
                Err(e) => Err(SearchIndexReadErr::DeserErr(e)),
            }
        }
        Err(_) => Err(SearchIndexReadErr::NotExistErr),
    }
}

pub async fn patch_search_from_update(note: &Note) {
    todo!();
    // let search_idx = read_search_index().await;
    // let doc_idx = read_doc_index().await;
    // let doc = tokenize_note_meta(note);
    // if let Some((search_idx, doc_idx)) = patch_search_index(doc, search_idx, doc_idx).await {
    //     write_search_index(&search_idx);
    // }
}

type Title = String;
type Content = String;
type ArchivePatch = (Title, Content);

pub async fn patch_search_from_archive(patch: ArchivePatch) {
    todo!();
    // let search_idx = read_search_index().await;
    // let doc_idx = read_doc_index().await;
    // let tokens = tokenize(&patch.1);
    // let doc = Doc {
    //     id: patch.0,
    //     tokens,
    //     content: patch.1,
    // };
    // if let Some((search_idx, doc_idx)) = patch_search_index(doc, search_idx, doc_idx).await {
    //     write_search_index(&search_idx);
    // }
}

pub async fn delete_entry_from_update(entry: &str) {
    todo!();
    // let search_idx = read_search_index().await;
    // let doc_idx = read_doc_index().await;
    // let (search_idx, doc_idx) = delete_entry_from_index(search_idx, doc_idx, entry).await;
    // write_search_index(&search_idx);
}

pub async fn delete_archived_file(entry: &str) {
    let mut archive_path = get_archive_location();
    archive_path.push(entry);
    if archive_path.exists() {
        remove_file(archive_path)
            .await
            .expect("Could not delete archive file");
    }
}

async fn delete_entry_from_index(
    mut search_idx: SearchIdx,
    mut doc_idx: DocIdx,
    entry: &str,
) -> (SearchIdx, DocIdx) {
    let doc = doc_idx
        .get(entry)
        .unwrap_or_else(|| panic!("Could not find doc marked for removal -- {}", entry));
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
    todo!();
    // let mut removed_tokens = Vec::new();
    // let mut added_tokens = Vec::new();
    // // TODO: Don't clone so much
    // if let Some(old_version) = doc_idx.get_mut(&doc.id) {
    //     let old_tokens = old_version.tokens.clone();
    //     for token in old_tokens.keys() {
    //         if !doc.tokens.keys().any(|f| f == token) {
    //             removed_tokens.push(token);
    //         }
    //     }
    //     for token in doc.tokens.keys() {
    //         if !old_tokens.keys().any(|f| f == token) {
    //             added_tokens.push(token)
    //         }
    //     }

    //     for token in removed_tokens {
    //         old_version.tokens.remove(token).unwrap();
    //         if let Some(search_token) = search_idx.get_mut(token) {
    //             *search_token = search_token
    //                 .iter()
    //                 .filter(|&f| f != &doc.id)
    //                 .map(|t| t.to_owned())
    //                 .collect::<Vec<String>>();
    //             if search_token.is_empty() {
    //                 search_idx.remove(token).unwrap();
    //             }
    //         }
    //     }

    //     for token in added_tokens {
    //         let doc_id = doc.id.clone();
    //         if let Some(search_token) = old_version.tokens.get_mut(token) {
    //             *search_token += 1;
    //         } else {
    //             old_version.tokens.insert(token.clone(), 1);
    //         }
    //         if let Some(search_token) = search_idx.get_mut(token) {
    //             search_token.push(doc_id);
    //         } else {
    //             search_idx.insert(token.clone(), vec![doc_id]);
    //         }
    //     }
    //     doc_idx.insert(doc.id.clone(), doc);
    //     Some((search_idx, doc_idx))
    // } else {
    //     for token in doc.tokens.keys() {
    //         let doc_id = doc.id.clone();
    //         if let Some(search_token) = search_idx.get_mut(token) {
    //             search_token.push(doc_id);
    //         } else {
    //             search_idx.insert(token.clone(), vec![doc_id]);
    //         }
    //     }
    //     doc_idx.insert(doc.id.clone(), doc);
    //     Some((search_idx, doc_idx))
    // }
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
        let (new_search, new_docs) = patch_search_index(updated_doc, search_idx, doc_idx)
            .await
            .unwrap();

        let added_doc = Doc {
            id: "added_doc".into(),
            tokens: HashMap::from([("added".into(), 1), ("doc".into(), 1)]),
            content: "added doc".into(),
        };

        let (new_search, new_docs) = patch_search_index(added_doc.clone(), new_search, new_docs)
            .await
            .unwrap();
        let updated_search_term_info = new_search.get("info");
        let updated_search_term_cool = new_search.get("cool");
        let updated_doc_id_added = new_docs.get("added_doc");
        let search_term_test = new_search.get("test");
        let search_term_token = new_search.get("token");
        assert_eq!(updated_search_term_info, Some(&vec!["test_doc".into()]));
        assert_eq!(updated_search_term_cool, Some(&vec!["test_doc".into()]));
        assert_eq!(search_term_test, Some(&vec!["another_doc".into()]));
        assert_eq!(search_term_token, None);
        assert!(updated_doc_id_added.is_some());
        let updated_doc_added = updated_doc_id_added.unwrap();
        assert_eq!(updated_doc_added.id, added_doc.id);
        assert_eq!(updated_doc_added.content, added_doc.content);
        assert_eq!(updated_doc_added.tokens, added_doc.tokens);
    }
}
