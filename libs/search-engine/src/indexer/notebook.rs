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
        // For some reason using tokio::read_dir never returns in the while loop
        let entries = read_dir(location).unwrap();
        entries.for_each(|entry| {
            let entry = entry.unwrap();
            if let Some(fname) = entry.file_name().to_str() {
                if fname.ends_with(".txt") {
                    let title = fname.strip_suffix(".txt").unwrap();
                    let content = path_to_string(&entry.path()).unwrap();
                    for (idx, line) in content.lines().enumerate() {
                        let raw_tokens = tokenize(line);
                        for token in raw_tokens {
                            tokens
                                .entry(token)
                                .and_modify(|v| v.push((title.to_string(), idx)))
                                .or_insert(vec![(title.to_string(), idx)]);
                        }
                    }
                }
            }
        });
        self.tokens = tokens;
    }
}
