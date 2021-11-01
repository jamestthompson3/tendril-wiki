use std::{collections::HashMap, path::Path};

use tasks::path_to_reader;

use crate::processors::tags::TagsArray;

#[derive(Debug)]
pub struct EditPageData {
    pub body: String,
    pub tags: Vec<String>,
    pub title: String,
    pub old_title: String,
    pub metadata: HashMap<String, String>,
}

impl From<HashMap<String, String>> for EditPageData {
    fn from(form_body: HashMap<String, String>) -> Self {
        let mut title: String = String::new();
        let mut old_title: String = String::new();
        let mut tags: Vec<String> = Vec::new();
        let mut body: String = String::new();
        let mut metadata: HashMap<String, String> = HashMap::new();
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
        EditPageData {
            body,
            tags,
            title,
            old_title,
            metadata,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum MetaParserState {
    Ready,
    Parsing,
    End,
}

#[derive(Debug, Default)]
pub struct NoteMeta {
    pub metadata: HashMap<String, String>,
    pub content: String,
}

#[derive(Copy, Clone)]
struct MetaParserMachine {
    state: MetaParserState,
}

impl MetaParserMachine {
    pub fn new() -> Self {
        MetaParserMachine {
            state: MetaParserState::Ready,
        }
    }

    pub fn send(&mut self, next_state: MetaParserState) {
        self.state = next_state;
    }

    pub fn current_state(self) -> MetaParserState {
        self.state
    }
}

impl From<EditPageData> for NoteMeta {
    fn from(data: EditPageData) -> Self {
        let mut metadata: HashMap<String, String> = data.metadata;
        metadata.insert("title".into(), data.title);
        let tags = TagsArray::from(data.tags);
        metadata.insert("tags".into(), tags.write());
        NoteMeta {
            metadata,
            content: data.body,
        }
    }
}

impl From<String> for NoteMeta {
    fn from(stringified: String) -> Self {
        parse_meta(stringified.lines().map(|l| l.into()), "raw_string") // mark that we've parsed from a passed string instead of a file
    }
}

impl Into<String> for NoteMeta {
    fn into(self) -> String {
        let mut formatted_string = String::from("---\n");
        for key in self.metadata.keys() {
            formatted_string.push_str(key);
            formatted_string.push_str(": ");
            formatted_string.push_str(self.metadata.get(key).unwrap());
            formatted_string.push('\n');
        }
        formatted_string.push_str("---\n");
        formatted_string.push_str(&self.content);
        formatted_string
    }
}

pub fn path_to_data_structure(path: &Path) -> Result<NoteMeta, Box<dyn std::error::Error>> {
    let reader = path_to_reader(path)?;
    Ok(parse_meta(reader, path.to_str().unwrap()))
}

pub fn parse_meta(lines: impl Iterator<Item = String>, debug_marker: &str) -> NoteMeta {
    let mut parser = MetaParserMachine::new();
    let mut notemeta = NoteMeta::default();
    for line in lines {
        match line.as_str() {
            "---" => match parser.current_state() {
                MetaParserState::Ready => parser.send(MetaParserState::Parsing),
                MetaParserState::Parsing => parser.send(MetaParserState::End),
                _ => {}
            },
            _ => match parser.current_state() {
                MetaParserState::Parsing => {
                    let values: Vec<&str> = line.split(": ").collect();
                    let vals: String;
                    assert!(values.len() > 1, "{}", debug_marker);
                    if values.len() > 2 {
                        vals = values[1..].join(": ");
                    } else {
                        vals = values[1].into()
                    }
                    notemeta.metadata.insert(values[0].into(), vals);
                }
                MetaParserState::End => {
                    notemeta.content.push_str(&format!("\n{}", line));
                }
                _ => {}
            },
        }
    }
    notemeta
}
