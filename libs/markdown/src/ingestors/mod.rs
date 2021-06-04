pub mod fs;

pub use self::fs::*;

use std::collections::HashMap;

#[derive(Debug)]
pub struct EditPageData {
    pub body: String,
    pub tags: Vec<String>,
    pub title: String,
    pub old_title: String,
}

impl From<HashMap<String, String>> for EditPageData {
    fn from(form_body: HashMap<String, String>) -> Self {
        let mut title: String = String::from("");
        let mut old_title: String = String::from("");
        let mut tags: Vec<String> = Vec::new();
        let mut body: String = String::from("");
        for key in form_body.keys() {
            match key.as_str() {
                "title" => title = form_body.get(key).unwrap().to_owned(),
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
                _ => {}
            }
        }
        EditPageData {
            body,
            tags,
            title,
            old_title,
        }
    }
}
