use build::get_data_dir_location;
use searcher::search;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, usize};

/// Heavy inspiration / code taken from: https://github.com/thesephist/monocle
use tokio::fs::write;

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
    let stored_location = get_data_dir_location();
    let mut p = Notebook::default();
    println!("Indexing notes...");
    p.load(&location).await;
    let index = p.index();
    println!("Writing persistent index...");
    write(
        stored_location.join("search-index.json"),
        serde_json::to_string(&index).unwrap(),
    )
    .await
    .unwrap();
    println!("Writing document files...");
    write(
        stored_location.join("docs.json"),
        serde_json::to_string(&p.docs_to_id()).unwrap(),
    )
    .await
    .unwrap();
}

pub async fn semantic_search(term: &str) -> Vec<(String, String)> {
    let index_location = get_data_dir_location();
    let results = search(term, index_location).await;
    results
        .into_iter()
        .map(|d| (d.id, d.content))
        .collect::<Vec<(String, String)>>()
}
