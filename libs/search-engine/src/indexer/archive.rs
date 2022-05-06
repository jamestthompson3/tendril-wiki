use compression::prelude::*;
use std::{fs::read_dir, path::Path};

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
                        let content = fs::read(entry.path()).await.unwrap();
                        let decompressed = content
                            .iter()
                            .cloned()
                            .decode(&mut BZip2Decoder::new())
                            .collect::<Result<Vec<_>, _>>()
                            .unwrap();
                        let text_content = String::from_utf8(decompressed).unwrap_or_else(|_| {
                            panic!(
                                "Unable to convert compressed text to utf8 string, {}",
                                fname
                            );
                        });
                        let tokenized_content = tokenize(&text_content);
                        let doc = Doc {
                            id: format!("{}.archive", fname),
                            tokens: tokenized_content,
                            content: text_content,
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
}
