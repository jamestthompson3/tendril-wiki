use compression::prelude::*;
use std::{
    fs::{read, read_dir},
    path::Path,
};

use crate::{tokenizer::tokenize, Doc};

use super::Proccessor;

#[derive(Default, Debug)]
pub(crate) struct Archive {
    pub(crate) documents: Vec<Doc>,
}

impl Proccessor for Archive {
    fn load(&mut self, location: &Path) {
        let entries = read_dir(location).unwrap();
        self.documents = entries
            .filter_map(|entry| {
                if let Ok(..) = entry {
                    let entry = entry.unwrap();
                    if let Some(fname) = entry.file_name().to_str() {
                        let content = read(entry.path()).unwrap();
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
            .collect::<Vec<Doc>>();
    }
}
