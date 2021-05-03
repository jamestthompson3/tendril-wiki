use std::{
    collections::HashMap,
    fmt::Write,
    fs::{write, File},
    io::{BufRead, BufReader},
    path::PathBuf,
};

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
    pub fn send(&mut self, next_state: MetaParserState) {
        self.state = next_state;
    }
    pub fn current_state(self) -> MetaParserState {
        self.state
    }
}

impl From<String> for NoteMeta {
    fn from(stringified: String) -> Self {
        parse_meta(stringified.lines().map(|s| s.to_string()), "raw_string") // mark that we've parsed from a passed string instead of a file
    }
}

impl Into<String> for NoteMeta {
    fn into(self) -> String {
        let mut metadata = String::new();
        for key in self.metadata.keys() {
            metadata.push_str(key);
            metadata.push_str(": ");
            metadata.push_str(self.metadata.get(key).unwrap());
            metadata.push_str("\n");
        }
        let mut formatted_string = String::new();
        writeln!(&mut formatted_string, "---").unwrap();
        writeln!(&mut formatted_string, "{}---", metadata).unwrap();
        writeln!(&mut formatted_string, "{}", self.content).unwrap();
        formatted_string
    }
}

pub fn path_to_data_structure(path: &PathBuf) -> Result<NoteMeta, Box<dyn std::error::Error>> {
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
                    assert_eq!(values.len() > 1, true, "{}", debug_marker);
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

/// Zombie utils that are useful when migrating from other formats
/// Could be part of the config file if it is useful
///
#[allow(dead_code)]
pub fn fix_tiddlywiki_tag_structures(path: &PathBuf) {
    let fd = File::open(&path).unwrap();
    let reader = BufReader::new(fd);
    let mut parser = MetaParserMachine::new();
    let mut fixed = String::new();
    for line in reader.lines() {
        let line_result = line.unwrap();
        let refline = line_result.as_str();
        match refline {
            "---" => match parser.current_state() {
                MetaParserState::Ready => {
                    parser.send(MetaParserState::Parsing);
                    fixed.push_str(&refline);
                }
                MetaParserState::Parsing => {
                    parser.send(MetaParserState::End);
                    fixed.push_str(&format!("\n{}", refline));
                }
                _ => {}
            },
            _ => match parser.current_state() {
                MetaParserState::Parsing => {
                    let values: Vec<&str> = refline.split(": ").collect();
                    if values[0] == "tags" {
                        let tags_first_pass: String = values[1]
                            .split("]]")
                            .map(|s| {
                                if s.starts_with("[[") {
                                    s.strip_prefix("[[").unwrap();
                                }
                                if s.ends_with("]]") {
                                    s.strip_suffix("]]").unwrap();
                                }
                                s
                            })
                            .collect::<Vec<&str>>()
                            .join("");
                        let tags = tags_first_pass
                            .split("[[")
                            .filter(|s| !s.is_empty() && s != &" ")
                            .map(|s| {
                                println!("{}", s);
                                s.trim()
                            })
                            .collect::<Vec<&str>>();
                        let mut fixed_string = String::from("[");
                        tags.iter().enumerate().for_each(|(idx, tag)| {
                            if idx != tags.len() - 1 {
                                fixed_string.push_str(&format!("{},", tag));
                            } else {
                                fixed_string.push_str(&format!("{}", tag));
                            }
                        });
                        fixed_string.push_str("]");
                        fixed.push_str(&format!("\ntags: {}", fixed_string));
                    } else {
                        fixed.push_str(&format!("\n{}", refline));
                    }
                }
                MetaParserState::End => {
                    fixed.push_str(&format!("\n{}", refline));
                }
                _ => {
                    fixed.push_str(&format!("\n{}", refline));
                }
            },
        }
    }
    write(&path, fixed).unwrap();
}

#[allow(dead_code)]
pub fn fix_tags(path: &PathBuf) {
    let fd = File::open(&path).unwrap();
    let reader = BufReader::new(fd);
    let mut parser = MetaParserMachine::new();
    let mut fixed = String::new();
    for line in reader.lines() {
        let line_result = line.unwrap();
        let refline = line_result.as_str();
        match refline {
            "---" => match parser.current_state() {
                MetaParserState::Ready => {
                    parser.send(MetaParserState::Parsing);
                    fixed.push_str(&refline);
                }
                MetaParserState::Parsing => {
                    parser.send(MetaParserState::End);
                    fixed.push_str(&format!("\n{}", refline));
                }
                _ => {}
            },
            _ => match parser.current_state() {
                MetaParserState::Parsing => {
                    let values: Vec<&str> = refline.split(": ").collect();
                    if values[0] == "tags" {
                        let mut fixed_string = String::from("[");
                        let tags: Vec<&str> = values[1]
                            .strip_prefix('[')
                            .unwrap()
                            .strip_suffix(']')
                            .unwrap()
                            .split(',')
                            .filter(|s| !s.is_empty() && s != &" ")
                            .map(|s| s.trim())
                            .map(|s| s.strip_prefix('"').unwrap().strip_suffix('"').unwrap())
                            .collect();
                        tags.iter().enumerate().for_each(|(idx, tag)| {
                            if idx != tags.len() - 1 {
                                fixed_string.push_str(&format!("{},", tag));
                            } else {
                                fixed_string.push_str(&format!("{}", tag));
                            }
                        });
                        fixed_string.push_str("]");
                        fixed.push_str(&format!("\ntags: {}", fixed_string));
                    } else {
                        fixed.push_str(&format!("\n{}", refline));
                    }
                }
                MetaParserState::End => {
                    fixed.push_str(&format!("\n{}", refline));
                }
                _ => {
                    fixed.push_str(&format!("\n{}", refline));
                }
            },
        }
    }
    write(&path, fixed).unwrap();
}
