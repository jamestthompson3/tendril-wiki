use compression::prelude::*;
use std::{
    collections::HashMap,
    fs::{read, read_dir},
    path::Path,
};

use crate::{tokenizer::tokenize, Doc, Tokens};

use super::Proccessor;

#[derive(Default, Debug)]
pub(crate) struct Archive {
    pub(crate) tokens: Tokens,
}

impl Proccessor for Archive {
    fn load(&mut self, location: &Path) {
        let entries = read_dir(location).unwrap();
        let mut tokens: Tokens = HashMap::new();
        let mut doc_token_counter: HashMap<String, f32> = HashMap::new();
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
                let mut total_tokens = 0;
                for line in text_content.lines() {
                    let raw_tokens = tokenize(line);
                    total_tokens += raw_tokens.len();
                    for token in raw_tokens {
                        doc_token_counter
                            .entry(token)
                            .and_modify(|v| *v += 1.)
                            .or_insert(1.);
                    }
                    for (term, count) in doc_token_counter.iter() {
                        tokens
                            .entry(term.to_owned())
                            .and_modify(|v| {
                                v.push((fname.to_string(), *count / total_tokens as f32))
                            })
                            .or_insert(vec![(fname.to_string(), *count / total_tokens as f32)]);
                    }
                    doc_token_counter.clear();
                }
            }
        });
    }
}
