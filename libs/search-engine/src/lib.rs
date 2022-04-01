use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap, path::PathBuf, usize};

/// Heavy inspiration / code taken from: https://github.com/thesephist/monocle
use tokenizer::tokenize;
use tokio::fs::{read_dir, read_to_string, write};

mod tokenizer;

#[derive(Serialize, Deserialize)]
struct Doc<'a> {
    tokens: HashMap<String, usize>,
    content: Cow<'a, String>,
    title: Option<Cow<'a, String>>,
    href: Option<Cow<'a, String>>,
}

#[async_trait]
pub(crate) trait Proccessor {
    fn tokenize(&self) -> Vec<HashMap<String, usize>>;
    async fn load(&mut self, location: PathBuf);
}

struct Notebook<'a> {
    pub(crate) documents: Vec<Doc<'a>>,
}

#[async_trait]
impl Proccessor for Notebook<'_> {
    fn tokenize(&self) -> Vec<HashMap<String, usize>> {
        self.documents
            .iter()
            .map(|d| d.tokens.to_owned())
            .collect::<Vec<HashMap<String, usize>>>()
    }
    async fn load(&'_ mut self, location: PathBuf) {
        let mut entries = read_dir(location).await.unwrap();
        while let Ok(entry) = entries.next_entry().await {
            if let Some(entry) = entry {
                if let Some(fname) = entry.file_name().to_str() {
                    if fname.ends_with(".md") {
                        let content = read_to_string(entry.path()).await.unwrap();
                        let tokenized_entry = tokenize(&content);
                        let doc = Doc {
                            tokens: tokenized_entry,
                            content: Cow::Owned(content),
                            title: None,
                            href: None,
                        };
                        self.documents.push(doc);
                    }
                }
            }
        }
    }
}
pub async fn build_search_index(location: PathBuf) {
    let mut p = Notebook {
        documents: Vec::new(),
    };
    p.load(location).await;
    write(
        "./search-index.json",
        serde_json::to_string(&p.tokenize()).unwrap(),
    )
    .await
    .unwrap();
}

// pub fn process(slice: &str) -> Vec<String> {
//     let tokens = tokenize(slice);
//     let mut stemmed_keys = tokens
//         .keys()
//         .map(|k| stem::get(k).unwrap())
//         .collect::<Vec<String>>();
//     stemmed_keys.sort_by_key(|a| a.len());
//     stemmed_keys
// }
