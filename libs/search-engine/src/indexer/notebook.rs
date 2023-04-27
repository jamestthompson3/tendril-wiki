use super::Proccessor;
use crate::{tokenizer::tokenize, Tokens};
use persistance::fs::path_to_string;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::read_dir, path::Path};

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct Notebook {
    pub(crate) tokens: Tokens,
}

impl Proccessor for Notebook {
    fn load(&mut self, location: &Path) {
        let mut tokens: Tokens = HashMap::new();
        let mut doc_token_counter: HashMap<String, f32> = HashMap::new();
        // For some reason using tokio::read_dir never returns in the while loop
        let entries = read_dir(location).unwrap();
        entries.for_each(|entry| {
            let entry = entry.unwrap();
            if let Some(fname) = entry.file_name().to_str() {
                if fname.ends_with(".txt") {
                    let title = fname.strip_suffix(".txt").unwrap();
                    let content = path_to_string(&entry.path()).unwrap();
                    let mut total_tokens = 0;
                    for line in content.lines() {
                        let raw_tokens = tokenize(line);
                        total_tokens += raw_tokens.len();
                        for token in raw_tokens {
                            doc_token_counter
                                .entry(token)
                                .and_modify(|v| *v += 1.)
                                .or_insert(1.);
                        }
                    }
                    for (term, count) in doc_token_counter.iter() {
                        tokens
                            .entry(term.to_owned())
                            .and_modify(|v| v.push((title.to_string(), *count / total_tokens as f32)))
                            .or_insert(vec![(title.to_string(), *count / total_tokens as f32)]);
                    }
                    doc_token_counter.clear();
                }
            }
        });
        self.tokens = tokens;
    }
}
