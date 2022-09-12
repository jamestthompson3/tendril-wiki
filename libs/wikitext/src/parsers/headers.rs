use std::collections::HashMap;
use std::fmt::Write as _;

use tasks::messages::PatchData;

use crate::processors::tags::TagsArray;

#[derive(Copy, Clone, PartialEq, Debug)]
enum MetaParserState {
    Parsing,
    End,
}

#[derive(Debug, Default)]
pub struct Note {
    pub header: HashMap<String, String>,
    pub content: String,
}

#[derive(Copy, Clone)]
struct HeaderParserMachine {
    state: MetaParserState,
}

impl HeaderParserMachine {
    pub fn new() -> Self {
        HeaderParserMachine {
            state: MetaParserState::Parsing,
        }
    }

    pub fn send(&mut self, next_state: MetaParserState) {
        self.state = next_state;
    }

    pub fn current_state(self) -> MetaParserState {
        self.state
    }
}

impl From<PatchData> for Note {
    fn from(data: PatchData) -> Self {
        let mut metadata: HashMap<String, String> = data.metadata;
        metadata.insert("title".into(), data.title);
        let tags = TagsArray::from(data.tags);
        metadata.insert("tags".into(), tags.write());
        Note {
            header: metadata,
            content: data.body,
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<PatchData> for Note {
    fn into(self) -> PatchData {
        let title = self.header.get("title").unwrap().to_owned();
        let tags = self.header.get("tags").unwrap().to_owned();
        let old_title = title.clone();
        PatchData {
            body: self.content,
            tags: TagsArray::from(tags).values,
            title,
            old_title,
            metadata: self.header,
        }
    }
}

impl From<&PatchData> for Note {
    fn from(data: &PatchData) -> Self {
        let mut metadata: HashMap<String, String> = data.metadata.clone();
        metadata.insert("title".into(), data.title.clone());
        let tags = TagsArray::from(data.tags.clone());
        metadata.insert("tags".into(), tags.write());
        Note {
            header: metadata,
            content: data.body.clone(),
        }
    }
}

impl From<String> for Note {
    fn from(stringified: String) -> Self {
        parse_meta(stringified.lines(), "raw_string") // mark that we've parsed from a passed string instead of a file
    }
}

#[allow(clippy::from_over_into)]
impl Into<String> for Note {
    fn into(self) -> String {
        let mut formatted_string = String::new();
        for key in self.header.keys() {
            formatted_string.push_str(key);
            formatted_string.push_str(": ");
            formatted_string.push_str(self.header.get(key).unwrap());
            formatted_string.push('\n');
        }
        formatted_string.push('\n');
        formatted_string.push_str(&self.content);
        formatted_string
    }
}

pub fn parse_meta<'a>(lines: impl Iterator<Item = &'a str>, debug_marker: &str) -> Note {
    let mut parser = HeaderParserMachine::new();
    let mut notemeta = Note::default();
    for line in lines {
        if line.is_empty() {
            if parser.current_state() == MetaParserState::Parsing {
                parser.send(MetaParserState::End);
            }
            continue;
        } else {
            match parser.current_state() {
                MetaParserState::Parsing => {
                    let values: Vec<&str> = line.split(": ").collect();
                    assert!(values.len() > 1, "{} --> {:?}", debug_marker, values);
                    let vals = if values.len() > 2 {
                        values[1..].join(": ")
                    } else {
                        values[1].into()
                    };
                    notemeta.header.insert(values[0].into(), vals);
                }
                MetaParserState::End => {
                    if notemeta.content.is_empty() {
                        write!(notemeta.content, "{}", line).unwrap();
                    } else {
                        write!(notemeta.content, "\n{}", line).unwrap();
                    }
                }
            }
        }
    }
    notemeta
}
