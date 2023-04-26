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
                for (idx, line) in text_content.lines().enumerate() {
                    let raw_tokens = tokenize(line);
                    for token in raw_tokens {
                        tokens
                            .entry(token)
                            .and_modify(|v| v.push((fname.to_string(), idx)))
                            .or_insert(vec![(fname.to_string(), idx)]);
                    }
                }
            }
        });
    }
}
