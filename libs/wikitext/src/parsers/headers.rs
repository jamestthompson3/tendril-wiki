use std::collections::HashMap;
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::processors::tags::{tag_string_from_vec, TagsArray};
use crate::PatchData;

use super::{get_outlinks, to_html, Html, ParsedTemplate, TemplattedPage};

#[derive(Copy, Clone, PartialEq, Debug)]
enum MetaParserState {
    Parsing,
    End,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Note {
    pub header: HashMap<String, String>,
    pub content: String,
}

#[derive(Debug, Default, Clone)]
pub struct StructuredNote<'a> {
    pub title: &'a str,
    pub links_and_tags: Vec<&'a str>,
}

impl StructuredNote<'_> {
    pub fn as_owned(&self) -> (String, Vec<String>) {
        (
            self.title.to_string(),
            self.links_and_tags.iter().map(|l| l.to_string()).collect(),
        )
    }
}

impl Note {
    fn parse_tags(&self) -> Vec<&str> {
        match self.header.get("tags") {
            None => Vec::with_capacity(0),
            Some(raw_tags) => TagsArray::new(raw_tags).values,
        }
    }
    pub fn to_template(&self) -> ParsedTemplate {
        let content_type = if let Some(content_type) = self.header.get("content-type") {
            content_type.as_str()
        } else {
            "text"
        };
        let html = if content_type == "html" {
            Html {
                body: self.content.clone(),
                outlinks: Vec::with_capacity(0),
            }
        } else {
            to_html(&self.content)
        };
        let title = self.header.get("title").unwrap();
        let tags = self.parse_tags();
        let mut rendered_metadata = self.header.to_owned();
        // We're already showing this, so no need to dump it in the table...
        rendered_metadata.remove("title");
        rendered_metadata.remove("tags");
        let desc = if self.content.len() >= 100 {
            if content_type != "html" {
                let mut shortened_desc = self.content.clone();
                shortened_desc.truncate(80);
                shortened_desc.push_str("...");
                shortened_desc
            } else {
                title.to_string()
            }
        } else {
            self.content.clone()
        };
        let page = TemplattedPage {
            title: title.to_string(),
            tags: tags.into_iter().map(|t| t.to_string()).collect(),
            body: html.body,
            metadata: rendered_metadata,
            desc,
        };
        ParsedTemplate {
            outlinks: html.outlinks.into_iter().map(|t| t.to_string()).collect(),
            page,
        }
    }
    pub fn to_structured(&self) -> StructuredNote {
        let mut links = get_outlinks(&self.content);
        links.extend(self.parse_tags());
        StructuredNote {
            title: self.header.get("title").unwrap(),
            links_and_tags: links,
        }
    }
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
        metadata.insert("tags".into(), tag_string_from_vec(data.tags));
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
            tags: TagsArray::new(&tags)
                .values
                .iter()
                .map(|&t| t.to_owned())
                .collect::<Vec<String>>(),
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
        metadata.insert("tags".into(), tag_string_from_vec((*data.tags).to_vec()));
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
