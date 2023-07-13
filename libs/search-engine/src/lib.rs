use indexer::{notebook::Notebook, tokenize_document};
use persistance::fs::utils::{
    get_archive_location, get_search_file_index_location, get_search_index_location,
};
use searcher::search;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{create_dir, read, write},
    path::{Path, PathBuf},
    process::exit,
    usize,
};
use thiserror::Error;
use wikitext::parsers::Note;

use tokio::fs::remove_file;

use crate::indexer::{archive::Archive, Proccessor};

mod indexer;
mod searcher;
mod tokenizer;

type SearchTerm = String;
type DocTitle = String;
type NormalizedFrequency = f32;
pub type Tokens = HashMap<SearchTerm, Vec<(DocTitle, NormalizedFrequency)>>;

#[derive(Error, Debug)]
pub enum SearchIndexErr {
    #[error("Could not find file")]
    NotExistErr,
    #[error("Could not deserialize file")]
    DeserErr(bincode::Error),
    #[error("Could not write serialized file")]
    WriteErr(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct Doc {
    id: String,
    tokens: Tokens,
}

pub fn build_search_index(location: &str) {
    let loc = get_search_index_location();
    if !loc.exists() {
        create_dir(&loc).unwrap();
        create_dir(&get_search_file_index_location()).unwrap();
    }
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
    write_search_index(&n.tokens, vec![n.file_index, a.file_index]);
}

pub async fn semantic_search(term: &str) -> Vec<String> {
    search(term).await
}

pub(crate) fn write_search_index(
    search_idx: &Tokens,
    term_indicies: Vec<HashMap<DocTitle, Vec<SearchTerm>>>,
) {
    let loc = get_search_index_location();
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
    // write the term_index for easy deletion
    let term_index_loc = get_search_file_index_location();
    for index in term_indicies.iter() {
        for (file, terms) in index.iter() {
            let bytes = bincode::serialize(terms).unwrap();
            let index_loc = term_index_loc.join(file);
            match write(index_loc, bytes) {
                Ok(()) => {}
                Err(e) => {
                    eprintln!("Could not write file index -> {}\n{}", file, e);
                    exit(1);
                }
            }
        }
    }
}

fn read_file_term_index(location: &Path) -> Result<Vec<String>, SearchIndexErr> {
    let term_index_loc = get_search_file_index_location();
    match read(term_index_loc.join(location)) {
        Ok(content) => {
            let deserialized_terms = bincode::deserialize(&content);
            match deserialized_terms {
                Ok(terms) => Ok(terms),
                Err(e) => Err(SearchIndexErr::DeserErr(e)),
            }
        }
        Err(_) => Err(SearchIndexErr::NotExistErr),
    }
}

fn write_file_term_index(location: &Path, content: Vec<String>) -> Result<(), SearchIndexErr> {
    let serialized_terms = bincode::serialize(&content);
    match serialized_terms {
        Ok(terms) => {
            write(location, terms).unwrap();
            Ok(())
        }
        Err(e) => Err(SearchIndexErr::DeserErr(e)),
    }
}

pub(crate) fn read_search_index(
    filename: &str,
) -> Result<Vec<(DocTitle, NormalizedFrequency)>, SearchIndexErr> {
    let index_location = get_search_index_location();
    let read_loc = index_location.join(filename);
    match read(read_loc) {
        Ok(content) => {
            let deserialized_freqs = bincode::deserialize(&content);
            match deserialized_freqs {
                Ok(tokens) => Ok(tokens),
                Err(e) => Err(SearchIndexErr::DeserErr(e)),
            }
        }
        Err(_) => Err(SearchIndexErr::NotExistErr),
    }
}

pub fn patch_search_from_update(note: &Note) {
    let mut content = note.content.clone();
    let title = note.header.get("title").unwrap();
    content.push('\n');
    content.push_str(title);
    let doc_token_count = tokenize_document(content);
    patch(doc_token_count, title.to_owned());
}

pub fn patch(doc_token_count: HashMap<String, f32>, title: String) {
    let term_index_loc = get_search_file_index_location();
    let index_loc = term_index_loc.join(&title);
    let term_index_doc = read_file_term_index(&index_loc).unwrap();
    let mut file_terms = Vec::with_capacity(doc_token_count.len());
    for (term, score) in doc_token_count.iter() {
        file_terms.push(term.to_owned());
        if let Ok(mut tokens) = read_search_index(term) {
            let mut found = false;
            for data in tokens.iter_mut() {
                if data.0 == *title {
                    found = true;
                    *data = (title.clone(), *score);
                }
            }
            if !found {
                tokens.push((title.clone(), *score));
            }
            write_search_entry(term, &tokens).unwrap();
        } else {
            let tokens = vec![(title.to_owned(), *score)];
            // The term we've parsed doesn't yet exist.
            write_search_entry(term, &tokens).unwrap();
        }
    }
    for term in term_index_doc.iter() {
        if file_terms.contains(term) {
            continue;
        }

        let tokens = read_search_index(term).unwrap();
        let tokens = tokens.into_iter().filter(|t| t.0 != *title).collect();
        write_search_entry(term, &tokens).unwrap();
    }
    write_file_term_index(&index_loc, file_terms).unwrap();
}

type Title = String;
type Content = String;
type ArchivePatch = (Title, Content);

pub async fn patch_search_from_archive(archive_patch: ArchivePatch) {
    let content = [archive_patch.0.clone(), archive_patch.1].join("\n");
    let doc_token_count = tokenize_document(content);
    patch(doc_token_count, archive_patch.0);
}

fn write_search_entry(
    entry: &str,
    content: &Vec<(DocTitle, NormalizedFrequency)>,
) -> Result<(), SearchIndexErr> {
    let bytes = bincode::serialize(content);
    let path = get_search_index_location();
    match bytes {
        Ok(b) => match write(path.join(entry), b) {
            Ok(()) => Ok(()),
            Err(e) => Err(SearchIndexErr::WriteErr(format!(
                "Could not write {}\n  {}",
                entry, e
            ))),
        },
        Err(e) => Err(SearchIndexErr::DeserErr(e)),
    }
}

pub async fn delete_entry_from_update(entry: &str) {
    let search_file_idx = get_search_file_index_location();
    let entry_file = search_file_idx.join(entry);
    let entries = read_file_term_index(&entry_file).unwrap();
    for e in entries.iter() {
        let contents = read_search_index(e).unwrap();
        let filtered_contents = contents
            .into_iter()
            .filter(|c| c.0 == entry)
            .collect::<Vec<(String, f32)>>();
        let bytes = bincode::serialize(&filtered_contents).unwrap();
        write(e, bytes).unwrap();
    }
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
