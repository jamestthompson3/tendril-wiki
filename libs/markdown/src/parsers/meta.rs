use std::{collections::HashMap, path::Path};

use crate::{ingestors::EditPageData, processors::tags::TagsArray};

use super::path_to_reader;

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
    #[inline]
    pub fn send(&mut self, next_state: MetaParserState) {
        self.state = next_state;
    }
    #[inline]
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
