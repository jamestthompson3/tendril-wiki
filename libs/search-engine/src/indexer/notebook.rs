use std::{collections::HashMap, fs::read_dir, path::Path};

use async_trait::async_trait;
use futures::{stream, StreamExt};
use markdown::parsers::{to_html, NoteMeta};
use persistance::fs::path_to_data_structure;

use crate::{tokenizer::tokenize, Doc};

use super::Proccessor;


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
                            let mut content = path_to_data_structure(&entry.path()).await.unwrap();
                            if content.metadata.get("title").is_none() {
                                let fixed_name = fname.strip_suffix(".md").unwrap();
                                content
                                    .metadata
                                    .insert("title".into(), fixed_name.to_owned());
                            }

                            let doc = tokenize_note_meta(&content);
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
    fn docs_to_idx(&self) -> HashMap<String, &Doc> {
        self.documents
            .iter()
            .map(|doc| (doc.id.clone(), doc))
            .collect::<HashMap<_, _>>()
    }
}

pub(crate) fn tokenize_note_meta(content: &NoteMeta) -> Doc {
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
    // TODO: Continue to fine tune weighting for different aspects of the note
    let title_tokens = tokenize(title.unwrap());
    for token in title_tokens.keys() {
        if let Some(title_token) = tokenized_entry.get_mut(token) {
            *title_token *= 3;
        } else {
            println!("Failed to tokenize {} in {:?}", token, tokenized_entry);
        }
    }

    Doc {
        id: title.unwrap().to_owned(),
        tokens: tokenized_entry,
        content: to_html(&content.content).body,
    }
}
