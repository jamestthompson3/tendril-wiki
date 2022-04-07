use std::{collections::HashMap, fs::read_dir, path::Path};

use async_trait::async_trait;
use build::get_data_dir_location;
use futures::{stream, StreamExt};
use markdown::parsers::{path_to_data_structure, to_html};
use tokio::fs::read_to_string;

use crate::{tokenizer::tokenize, Doc};

#[async_trait]
pub(crate) trait Proccessor {
    async fn load(&mut self, location: &Path);
    fn index(&self) -> HashMap<String, Vec<String>>;
    fn docs_to_id(&self) -> HashMap<String, &Doc>;
}

#[derive(Default, Debug)]
pub(crate) struct Notebook {
    pub(crate) documents: Vec<Doc>,
}

#[async_trait]
impl Proccessor for Notebook {
    async fn load(&'_ mut self, location: &Path) {
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

pub(crate) async fn patch_search_index(doc: Doc) {
    let index_location = get_data_dir_location();
    let doc_idx = read_to_string(index_location.join("docs.json"))
        .await
        .unwrap();
    let mut doc_idx: HashMap<String, Doc> = serde_json::from_str(&doc_idx).unwrap();
    let search_idx = read_to_string(index_location.join("search-index.json"))
        .await
        .unwrap();
    let mut search_idx: HashMap<String, Vec<String>> = serde_json::from_str(&search_idx).unwrap();
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
    }
}
