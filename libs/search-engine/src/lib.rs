use async_trait::async_trait;
use futures::{stream, StreamExt};
use markdown::parsers::{path_to_data_structure, to_html};
use searcher::search;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap, fs::read_dir, path::PathBuf, usize};

/// Heavy inspiration / code taken from: https://github.com/thesephist/monocle
use tokenizer::tokenize;
use tokio::fs::write;

mod searcher;
mod tokenizer;

pub(crate) type Tokens = HashMap<String, usize>;

#[derive(Serialize, Deserialize, Debug)]
struct Doc<'a> {
    id: String,
    tokens: Tokens,
    content: String,
    title: Option<Cow<'a, String>>,
    href: Option<Cow<'a, String>>,
}

#[async_trait]
pub(crate) trait Proccessor {
    async fn load(&mut self, location: PathBuf);
    fn index(&self) -> HashMap<String, Vec<String>>;
    fn docs_to_id(&self) -> HashMap<String, &Doc>;
}

struct Notebook<'a> {
    pub(crate) documents: Vec<Doc<'a>>,
}

#[async_trait]
impl Proccessor for Notebook<'_> {
    async fn load(&'_ mut self, location: PathBuf) {
        // For some reason using tokio::read_dir never returns in the while loop
        let entries = read_dir(location).unwrap();
        self.documents = stream::iter(entries)
            .filter_map(|entry| async move {
                if let Ok(..) = entry {
                    let entry = entry.unwrap();
                    if let Some(fname) = entry.file_name().to_str() {
                        if fname.ends_with(".md") {
                            let content = path_to_data_structure(&entry.path()).await.unwrap();
                            let mut tokeniziable_content = content.content.clone();
                            let tags = content.metadata.get("tags");
                            let title = content.metadata.get("title");
                            // create space between content and tags
                            tokeniziable_content.push(' ');
                            tokeniziable_content.push_str(tags.unwrap_or(&String::from("")));
                            // create space between content and title
                            tokeniziable_content.push(' ');
                            tokeniziable_content.push_str(title.unwrap_or(&String::from("")));
                            let mut tokenized_entry = tokenize(&tokeniziable_content);
                            let id = if let Some(t) = title {
                                t.to_string()
                            } else {
                                let fixed_name = fname.strip_suffix(".md").unwrap();
                                fixed_name.to_owned()
                            };
                            // TODO: Continue to fine tune weighting for different aspects of the note
                            let title_tokens = tokenize(title.unwrap());
                            for token in title_tokens.keys() {
                                if let Some(title_token) = tokenized_entry.get_mut(token) {
                                    *title_token *= 3;
                                } else {
                                    println!(
                                        "Failed to tokenize {} in {:?}",
                                        token, tokenized_entry
                                    );
                                }
                            }

                            let doc = Doc {
                                id,
                                tokens: tokenized_entry,
                                content: to_html(&content.content).body,
                                title: None,
                                href: None,
                            };
                            Some(doc)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<Doc>>()
            .await;
    }
    fn index(&self) -> HashMap<String, Vec<String>> {
        let mut index = HashMap::with_capacity(self.documents.len());
        self.documents.iter().for_each(|doc| {
            let tokens = &doc.tokens;
            for key in tokens.keys() {
                if index.get(key).is_none() {
                    index.insert(key.to_owned(), vec![doc.id.to_owned()]);
                } else {
                    let ids = index.get_mut(key).unwrap();
                    ids.push(doc.id.to_owned());
                }
            }
        });
        index
    }
    fn docs_to_id(&self) -> HashMap<String, &Doc> {
        self.documents
            .iter()
            .map(|doc| (doc.id.clone(), doc))
            .collect::<HashMap<_, _>>()
    }
}

pub async fn build_search_index(location: PathBuf) {
    let mut p = Notebook {
        documents: Vec::new(),
    };
    println!("Indexing notes...");
    p.load(location).await;
    let index = p.index();
    println!("Writing persistent index...");
    write(
        "./search-index.json",
        serde_json::to_string(&index).unwrap(),
    )
    .await
    .unwrap();
    println!("Writing document files...");
    write(
        "./docs.json",
        serde_json::to_string(&p.docs_to_id()).unwrap(),
    )
    .await
    .unwrap();
}

pub async fn semantic_search(term: &str) -> Vec<(String, String)> {
    let results = search(term).await;
    results
        .into_iter()
        .map(|d| (d.id, d.content))
        .collect::<Vec<(String, String)>>()
}
