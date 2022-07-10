use std::collections::HashMap;
use std::fmt::Write as _;

use tasks::messages::PatchData;

use crate::processors::tags::TagsArray;

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

impl From<PatchData> for NoteMeta {
    fn from(data: PatchData) -> Self {
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

#[allow(clippy::from_over_into)]
impl Into<PatchData> for NoteMeta {
    fn into(self) -> PatchData {
        let title = self.metadata.get("title").unwrap().to_owned();
        let tags = self.metadata.get("tags").unwrap().to_owned();
        let old_title = title.clone();
        PatchData {
            body: self.content,
            tags: TagsArray::from(tags).values,
            title,
            old_title,
            metadata: self.metadata,
        }
    }
}

impl From<&PatchData> for NoteMeta {
    fn from(data: &PatchData) -> Self {
        let mut metadata: HashMap<String, String> = data.metadata.clone();
        metadata.insert("title".into(), data.title.clone());
        let tags = TagsArray::from(data.tags.clone());
        metadata.insert("tags".into(), tags.write());
        NoteMeta {
            metadata,
            content: data.body.clone(),
        }
    }
}

impl From<String> for NoteMeta {
    fn from(stringified: String) -> Self {
        parse_meta(stringified.lines(), "raw_string") // mark that we've parsed from a passed string instead of a file
    }
}

#[allow(clippy::from_over_into)]
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

pub fn parse_meta<'a>(lines: impl Iterator<Item = &'a str>, debug_marker: &str) -> NoteMeta {
    let mut parser = MetaParserMachine::new();
    let mut notemeta = NoteMeta::default();
    for line in lines {
        match line {
            "---" => match parser.current_state() {
                MetaParserState::Ready => parser.send(MetaParserState::Parsing),
                MetaParserState::Parsing => parser.send(MetaParserState::End),
                _ => {}
            },
            _ => match parser.current_state() {
                MetaParserState::Parsing => {
                    let values: Vec<&str> = line.split(": ").collect();
                    assert!(values.len() > 1, "{}", debug_marker);
                    let vals = if values.len() > 2 {
                        values[1..].join(": ")
                    } else {
                        values[1].into()
                    };
                    notemeta.metadata.insert(values[0].into(), vals);
                }
                MetaParserState::End => {
                    write!(notemeta.content, "\n{}", line).unwrap();
                }
                _ => {}
            },
        }
    }
    notemeta
}
