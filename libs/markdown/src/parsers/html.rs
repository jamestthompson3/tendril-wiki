use pulldown_cmark::{html, Event, Options, Parser};
use urlencoding::encode;

#[derive(Copy, Clone, PartialEq, Debug)]
enum ParserState<'a> {
    LinkStart,         // [
    LocationParsing(&'a str),   // [[
    LinkEnd,           // ]
    TranscludeStart,   // {
    TranscludeParsing, // {{
    TranscludeEnd,     // }
    Accept,            // default
}

pub struct Html {
    pub outlinks: Vec<String>,
    pub body: String,
}

#[derive(Clone)]
struct ParserMachine<'a> {
    state: ParserState<'a>,
    link_location: String,
    outlinks: Vec<String>,
}

// Note A -> Note B -> Note C

impl<'a> ParserMachine<'a> {
    pub fn new() -> Self {
        ParserMachine {
            state: ParserState::Accept,
            link_location: String::new(),
            outlinks: Vec::new(),
        }
    }
    pub fn current_state(&mut self) -> ParserState<'a> {
        self.state
    }
    pub fn set_state(&mut self, next: ParserState<'a>) {
        self.state = next;
    }
    #[inline]
    pub fn send<'b>(&mut self, message: ParserState<'b>) -> Event<'static> {
        match message {
            ParserState::Accept => {
                match self.current_state() {
                    ParserState::Accept => {
                        self.set_state(message);
                        Event::Text("".into())
                    }
                    ParserState::LinkEnd => {
                        self.set_state(message);
                        Event::Text("".into())
                    }
                    ParserState::LinkStart => {
                        // [checkout thing]
                        {
                            self.set_state(message);
                            Event::Text("".into())
                        }
                    }
                    ParserState::LocationParsing(_) => panic!("Unclosed link"), // [[kajdf akdj
                    ParserState::TranscludeStart => {
                        // {some brackets}
                        {
                            self.set_state(message);
                            Event::Text("".into())
                        }
                    }
                    ParserState::TranscludeParsing => panic!("Unclosed transclusion"),
                    ParserState::TranscludeEnd => {
                        self.set_state(message);
                        Event::Text("".into())
                    }
                }
            }

            ParserState::LinkStart => {
                match self.current_state() {
                    ParserState::Accept => {
                        self.set_state(message);
                        Event::Text("".into())
                    }
                    ParserState::LinkStart => {
                        self.set_state(ParserState::LocationParsing(""));
                        Event::Text("".into())
                    }
                    ParserState::LocationParsing(_) => panic!("Link parsing has already started"),
                    ParserState::LinkEnd => panic!("Can't start link immediately from ending"),
                    ParserState::TranscludeStart => {
                        // {[some brackets]}
                        {
                            self.set_state(message);
                            Event::Text("".into())
                        }
                    }
                    ParserState::TranscludeParsing => {
                        self.set_state(message);
                        Event::Text("".into())
                    }
                    ParserState::TranscludeEnd => {
                        panic!("Can't start link immediately from transclude ending")
                    }
                }
            }
            ParserState::LocationParsing(text) => match self.current_state() {
                ParserState::Accept => panic!("must start parsing a link first"),
                ParserState::LinkStart => {
                    self.set_state(message);
                    Event::Text("]".into())
                }
                ParserState::LocationParsing(_) => {
                    self.link_location.push_str(&text);
                    Event::Text("".into())
                }
                ParserState::LinkEnd => panic!("Must start parsing a link first"),
                ParserState::TranscludeStart => panic!("Must start parsing a link first"),
                ParserState::TranscludeParsing => panic!("Must start parsing a link first"),
                ParserState::TranscludeEnd => panic!("Must start parsing a link first"),
            },
            ParserState::LinkEnd => {
                match self.current_state() {
                    // event disregarded here
                    ParserState::Accept => {
                        self.set_state(ParserState::Accept);
                        Event::Text("".into())
                    }
                    ParserState::LinkStart => panic!("must start parsing a link before ending"),
                    ParserState::LocationParsing(_) => {
                        // Event disregarded here
                        {
                            self.set_state(message);
                            Event::Text("".into())
                        }
                    }
                    ParserState::LinkEnd => {
                        // Event disregarded here
                        let location: &str;
                        let text: &str;
                        let link_location = self.link_location.clone(); // TODO: figure out why I have immutable borrow issues when I don't clone this.
                        if link_location.contains('|') {
                            let split_vals: Vec<&str> = link_location.split('|').collect();
                            assert_eq!(
                                split_vals.len() < 3,
                                true,
                                "Malformed wiki link: {} ---> {:?}",
                                self.link_location,
                                split_vals
                            );
                            location = split_vals[1];
                            text = split_vals[0];
                        } else {
                            location = &link_location;
                            text = &link_location;
                        }
                        let href = format_links(location);
                        self.link_location.clear();
                        self.outlinks.push(location.to_string());
                        self.set_state(ParserState::Accept);
                        Event::Html(format!(r#"<a href="{}">{}</a>"#, href, text).into())
                    }
                    ParserState::TranscludeStart => panic!("Must start parsing a link first"),
                    ParserState::TranscludeParsing => panic!("Must start parsing a link first"),
                    ParserState::TranscludeEnd => panic!("Must start parsing a link first"),
                }
            }
            ParserState::TranscludeStart => {
                match self.current_state() {
                    ParserState::Accept => {
                        self.set_state(message);
                        Event::Text("".into())
                    }
                    ParserState::LinkStart => {
                        self.set_state(ParserState::Accept);
                        Event::Text("".into())
                    } // [{skjfkj}]
                    ParserState::LocationParsing(_) => Event::Text("".into()), // [[{some weird text}|https://example.com]]
                    ParserState::LinkEnd => panic!("Must close existing link"), // ]}
                    ParserState::TranscludeStart => Event::Text("".into()),
                    ParserState::TranscludeParsing => {
                        panic!("Must finish current transclusion first")
                    }
                    ParserState::TranscludeEnd => panic!("Must finish current transclusion first"),
                }
            }
            ParserState::TranscludeParsing => match self.current_state() {
                ParserState::Accept => panic!("Must start transclusion first"),
                ParserState::LinkStart => panic!("Must start transclusion first"),
                ParserState::LocationParsing(_) => panic!("Must start transclusion first"),
                ParserState::LinkEnd => panic!("Must start transclusion first"),
                ParserState::TranscludeStart => {
                    self.set_state(message);
                    Event::Text("".into())
                }
                ParserState::TranscludeParsing => Event::Text("".into()),
                ParserState::TranscludeEnd => panic!("Must start transclusion first"),
            },
            ParserState::TranscludeEnd => {
                match self.current_state() {
                    ParserState::Accept => panic!("Must start transclusion first"),
                    ParserState::LinkStart => {
                        self.set_state(ParserState::Accept);
                        Event::Text("".into())
                    } // [}
                    ParserState::LocationParsing(_) => Event::Text("".into()), // [[}my note}]]
                    ParserState::LinkEnd => panic!("must close the link!"), // ]} [[my note kjsdfkj kdjf ]}
                    ParserState::TranscludeStart => {
                        self.set_state(ParserState::Accept);
                        Event::Text("".into())
                    } // {}
                    ParserState::TranscludeParsing => {
                        self.set_state(message);
                        Event::Text("".into())
                    }
                    ParserState::TranscludeEnd => Event::Text("".into()),
                }
            }
        }
    }
}

pub fn to_html(md: &str) -> Html {
    let mut parser_machine = ParserMachine::new();

    let parser = Parser::new_ext(&md, Options::all()).map(|event| match event {
        Event::Text(text) => match &*text {
            "[" => parser_machine.send(ParserState::LinkStart),
            "]" => match parser_machine.current_state() {
                ParserState::LinkEnd => parser_machine.send(ParserState::LinkEnd),
                ParserState::LocationParsing(_) => parser_machine.send(ParserState::LinkEnd),
                ParserState::Accept => Event::Text(text),
                _ => {
                    println!("{:?}", parser_machine.current_state());
                    panic!("Impossible state reached for `]`");
                }
            },
            _ => match parser_machine.current_state() {
                ParserState::LocationParsing(_) => {
                    parser_machine.send(ParserState::LocationParsing(&text))
                }
                ParserState::LinkEnd => {
                    let _ = parser_machine.send(ParserState::Accept);
                    Event::Text(format!("]{}", text).into())
                }
                ParserState::LinkStart => {
                    let _ = parser_machine.send(ParserState::Accept);
                    Event::Text(format!("[{}", text).into())
                }
                _ => {
                    // TODO: custom url schemas?
                    if text.starts_with("http") {
                        return Event::Html(format!(r#"<a href="{}">{}</a>"#, text, text).into());
                    }
                    Event::Text(text)
                }
            },
        },
        _ => event,
    });
    let mut output = String::new();
    html::push_html(&mut output, parser);
    Html {
        body: output,
        outlinks: parser_machine.outlinks,
    }
}

pub fn format_links(link: &str) -> String {
    let proto_prefixes = link.split(':').collect::<Vec<&str>>();
    match proto_prefixes[0] {
        "http" | "https" => link.to_string(),
        "files" => {
            format!("/files/{}", encode(link.strip_prefix("files:").unwrap()))
        }
        _ => format!("/{}", encode(&link)), // HACK: deal with warp decoding this later
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_links_properly() {
        let http_link = "https://example.com";
        assert_eq!(
            String::from("https://example.com"),
            format_links(&http_link)
        );
        let wiki_page = "My Cool Page";
        assert_eq!(String::from("/My%20Cool%20Page"), format_links(&wiki_page));
    }
    #[test]
    fn parses_md_to_html_with_wikilinks() {
        let test_string = "# Title\n [[Some Page]]. Another thing\n * Hi\n * List\n * Output";
        let test_html = Html {
            outlinks: vec!["Some Page".into()],
            body: "<h1>Title</h1>\n<p><a href=\"/Some%20Page\">Some Page</a>. Another thing</p>\n<ul>\n<li>Hi</li>\n<li>List</li>\n<li>Output</li>\n</ul>\n".into()
        };
        let parsed = to_html(test_string);
        assert_eq!(parsed.outlinks, test_html.outlinks);
        assert_eq!(parsed.body, test_html.body);
    }
}
