use std::{path::Path, collections::HashMap};

use crate::tokenizer::tokenize;

pub(crate) mod archive;
pub(crate) mod notebook;

pub(crate) trait Proccessor {
    fn load(&mut self, location: &Path);
}
pub type DocTokenCount = HashMap<String, f32>;

pub fn tokenize_document(content: String) -> DocTokenCount {
    let mut token_counter: DocTokenCount = HashMap::new();
    let mut total_tokens = 0.0;
    for line in content.lines() {
        let raw_tokens = tokenize(line);
        total_tokens += raw_tokens.len() as f32;
        for token in raw_tokens {
            token_counter
                .entry(token)
                .and_modify(|v| *v += 1.0)
                .or_insert(1.0);
        }
    }
    for (_, val) in token_counter.iter_mut() {
        *val /= total_tokens;
    }
    token_counter
}
