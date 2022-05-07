use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PatchData {
    pub body: String,
    pub tags: Vec<String>,
    pub title: String,
    pub old_title: String,
    pub metadata: HashMap<String, String>,
}

impl From<HashMap<String, String>> for PatchData {
    fn from(form_body: HashMap<String, String>) -> Self {
        let mut title: String = String::new();
        let mut old_title: String = String::new();
        let mut tags: Vec<String> = Vec::new();
        let mut body: String = String::new();
        let mut metadata: HashMap<String, String> = HashMap::new();
        for key in form_body.keys() {
            match key.as_str() {
                "title" => title = form_body.get(key).unwrap().trim().to_owned(),
                "old_title" => {
                    if let Some(old_title_from_form) = form_body.get(key) {
                        old_title = old_title_from_form.to_owned()
                    }
                }
                "tags" => {
                    tags = form_body
                        .get(key)
                        .unwrap()
                        .split(',')
                        .map(|s| s.to_owned())
                        .collect()
                }
                "body" => body = form_body.get(key).unwrap().to_owned(),
                "metadata" => {
                    let stringified_meta = form_body.get(key).unwrap().to_owned();
                    let kv_pairs = stringified_meta.split('\n');
                    for pair_string in kv_pairs {
                        // Support metadata attributes with the : character.
                        let unpaired: Vec<&str> = pair_string.split(':').collect();
                        let key = unpaired[0].to_owned();
                        let value = unpaired[1..].join(":");
                        metadata.insert(key, value);
                    }
                }
                _ => {}
            }
        }
        PatchData {
            body,
            tags,
            title,
            old_title,
            metadata,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    Patch {
        patch: PatchData,
    },
    Rebuild,
    Delete {
        title: String,
    },
    Archive {
        url: String,
        title: String,
    },
    ArchiveMove {
        old_title: String,
        new_title: String,
    },
}
