use compression::prelude::*;
use std::{
    collections::HashMap,
    fs::{read, read_dir},
    path::Path,
};

use crate::Tokens;

use super::{tokenize_document, Proccessor};

#[derive(Default, Debug)]
pub(crate) struct Archive {
    pub(crate) tokens: Tokens,
    pub(crate) file_index: HashMap<String, Vec<String>>,
}

impl Proccessor for Archive {
    fn load(&mut self, location: &Path) {
        let entries = read_dir(location).unwrap();
        let mut tokens: Tokens = HashMap::new();
        let mut term_index: HashMap<String, Vec<String>> = HashMap::new();
        entries.for_each(|entry| {
            let entry = entry.unwrap();
            if let Some(fname) = entry.file_name().to_str() {
                if fname.ends_with("pdf") {
                    return;
                }
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
                let doc_token_counter = tokenize_document(text_content);
                for (term, score) in doc_token_counter.iter() {
                    tokens
                        .entry(term.to_owned())
                        .and_modify(|v| v.push((fname.to_string(), *score)))
                        .or_insert(vec![(fname.to_string(), *score)]);
                    term_index
                        .entry(fname.to_owned())
                        .and_modify(|v| v.push(term.clone()))
                        .or_insert(vec![term.clone()]);
                }
            }
        });
        self.tokens = tokens;
        self.file_index = term_index;
    }
}
