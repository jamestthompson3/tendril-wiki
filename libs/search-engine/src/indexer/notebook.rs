use super::{Proccessor, tokenize_document};
use crate::Tokens;
use persistance::fs::path_to_string;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::read_dir, path::Path};

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct Notebook {
    pub(crate) tokens: Tokens,
    // filename, Vec<search_terms>
    pub(crate) file_index: HashMap<String, Vec<String>>,
}

impl Proccessor for Notebook {
    fn load(&mut self, location: &Path) {
        let mut tokens: Tokens = HashMap::new();
        let mut term_index: HashMap<String, Vec<String>> = HashMap::new();
        let entries = read_dir(location).unwrap();
        entries.for_each(|entry| {
            let entry = entry.unwrap();
            if let Some(fname) = entry.file_name().to_str() {
                if fname.ends_with(".txt") {
                    let title = fname.strip_suffix(".txt").unwrap();
                    let content = path_to_string(&entry.path()).unwrap();
                    let doc_token_counter = tokenize_document(content);
                    for (term, score) in doc_token_counter.iter() {
                        tokens
                            .entry(term.to_owned())
                            .and_modify(|v| v.push((title.to_string(), *score)))
                            .or_insert(vec![(title.to_string(), *score)]);
                        term_index
                            .entry(fname.to_owned())
                            .and_modify(|v| v.push(term.clone()))
                            .or_insert(vec![term.clone()]);
                    }
                }
            }
        });
        self.tokens = tokens;
        self.file_index = term_index;
    }
}
