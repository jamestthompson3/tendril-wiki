pub mod fs;

pub use self::fs::*;

use std::collections::HashMap;

#[derive(Debug)]
pub struct WebFormData {
    pub body: String,
    pub tags: Vec<String>,
    pub title: String,
}

impl From<HashMap<String, String>> for WebFormData {
    fn from(form_body: HashMap<String, String>) -> Self {
        let mut title: String = String::from("");
        let mut tags: Vec<String> = Vec::new();
        let mut body: String = String::from("");
        for key in form_body.keys() {
            match key.as_str() {
                "title" => title = form_body.get(key).unwrap().to_owned(),
                "tags" => {
                    tags = form_body
                        .get(key)
                        .unwrap()
                        .split(',')
                        .map(|s| s.to_string())
                        .collect()
                }
                "body" => body = form_body.get(key).unwrap().to_owned(),
                _ => {}
            }
        }
        WebFormData { title, tags, body }
    }
}
