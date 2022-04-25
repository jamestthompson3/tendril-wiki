use std::{collections::HashMap, fs::read_dir, path::Path};

use async_trait::async_trait;
use futures::{stream, StreamExt};
use tokio::fs;

use crate::{tokenizer::tokenize, Doc};

use super::Proccessor;

#[derive(Default, Debug)]
pub(crate) struct Archive {
    pub(crate) documents: Vec<Doc>,
}

#[async_trait]
impl Proccessor for Archive {
    async fn load(&mut self, location: &Path) {
        let entries = read_dir(location).unwrap();
        self.documents = stream::iter(entries)
            .filter_map(|entry| async move {
                if let Ok(..) = entry {
                    let entry = entry.unwrap();
                    if let Some(fname) = entry.file_name().to_str() {
                        let content = fs::read_to_string(entry.path()).await.unwrap();
                        let tokenized_content = tokenize(&content);
                        let doc = Doc {
                            id: format!("{}.archive", fname),
                            tokens: tokenized_content,
                            content,
                        };
                        Some(doc)
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
