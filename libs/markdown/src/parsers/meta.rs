use std::{
    collections::HashMap,
    fs::{write, File},
    io::{BufRead, BufReader},
    path::PathBuf,
};

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

pub fn as_data_structure(path: &PathBuf) -> NoteMeta {
    let fd = File::open(&path).unwrap();
    let reader = BufReader::new(fd);
    let mut parser = MetaParserMachine::new();
    let mut notemeta = NoteMeta::default();
    for line in reader.lines() {
        let line_result = line.unwrap();
        let refline = line_result.as_str();
        match refline {
            "---" => match parser.current_state() {
                MetaParserState::Ready => parser.send(MetaParserState::Parsing),
                MetaParserState::Parsing => parser.send(MetaParserState::End),
                _ => {}
            },
            _ => match parser.current_state() {
                MetaParserState::Parsing => {
                    let values: Vec<&str> = refline.split(": ").collect();
                    let vals: String;
                    assert_eq!(values.len() > 1, true, "{:?}", path);
                    if values.len() > 2 {
                        vals = values[1..].join(": ");
                    } else {
                        vals = values[1].to_string()
                    }
                    notemeta.metadata.insert(values[0].to_string(), vals);
                }
                MetaParserState::End => {
                    notemeta.content.push_str(&format!("\n{}", refline));
                }
                _ => {}
            },
        }
    }
    notemeta
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
