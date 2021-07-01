use std::cell::RefCell;

use pulldown_cmark::{html, Event, Options, Parser};
use urlencoding::encode;

#[derive(PartialEq, Debug, Clone)]
enum ParserState<'a> {
    LinkStart,                                   // [
    LocationParsing(pulldown_cmark::CowStr<'a>), // [[
    LinkEnd,                                     // ]
    TranscludeStart,                             // {
    TranscludeParsing,                           // {{
    TranscludeEnd,                               // }
    Accept,                                      // default
}

pub struct Html {
    pub outlinks: Vec<String>,
    pub body: String,
}

#[derive(Clone)]
struct ParserMachine<'a> {
    link_location: RefCell<String>,
    outlinks: RefCell<Vec<String>>,
    state: ParserState<'a>,
}

impl<'a> ParserMachine<'a> {
    fn send(mut self, message: ParserState<'a>) -> Event {
        match (&message, self.state) {
            (ParserState::Accept, ParserState::Accept) => {
                self.state = message;
                Event::Text("".into())
            }
            (ParserState::Accept, ParserState::LinkEnd) => {
                self.state = message;
                Event::Text("".into())
            }

            (ParserState::Accept, ParserState::LinkStart) => {
                // [checkout thing]
                {
                    self.state = message;
                    Event::Text("".into())
                }
            }
            (ParserState::Accept, ParserState::LocationParsing(_)) => panic!("Unclosed link"), // [[kajdf akdj
            (ParserState::Accept, ParserState::TranscludeStart) => {
                // {some brackets}
                {
                    self.state = message;
                    Event::Text("".into())
                }
            }
            (ParserState::Accept, ParserState::TranscludeParsing) => {
                panic!("Unclosed transclusion")
            }
            (ParserState::Accept, ParserState::TranscludeEnd) => {
                self.state = message;
                Event::Text("".into())
            }

            (ParserState::LinkStart, ParserState::Accept) => {
                self.state = message;
                Event::Text("".into())
            }

            (ParserState::LinkStart, ParserState::LinkStart) => {
                self.state = ParserState::LocationParsing("".into());
                Event::Text("".into())
            }

            (ParserState::LinkStart, ParserState::LocationParsing(_)) => {
                panic!("Link parsing has already started")
            }

            (ParserState::LinkStart, ParserState::LinkEnd) => {
                panic!("Can't start link immediately from ending")
            }

            (ParserState::LinkStart, ParserState::TranscludeStart) => {
                // {[some brackets]}
                {
                    self.state = message;
                    Event::Text("".into())
                }
            }
            (ParserState::LinkStart, ParserState::TranscludeParsing) => {
                self.state = message;
                Event::Text("".into())
            }

            (ParserState::LinkStart, ParserState::TranscludeEnd) => {
                panic!("Can't start link immediately from transclude ending")
            }
            (ParserState::LocationParsing(_), ParserState::Accept) => {
                panic!("must start parsing a link first")
            }
            (ParserState::LocationParsing(text), ParserState::LinkStart) => {
                self.state = message.clone();
                Event::Text(text.to_owned())
            }
            (ParserState::LocationParsing(text), ParserState::LocationParsing(_)) => {
                self.link_location.borrow_mut().push_str(text);
                {
                    self.state = ParserState::LocationParsing(text.to_owned());
                    Event::Text("".into())
                }
            }
            (ParserState::LocationParsing(_), ParserState::LinkEnd) => {
                panic!("Must start parsing a link first")
            }
            (ParserState::LocationParsing(_), ParserState::TranscludeStart) => {
                panic!("Must start parsing a link first")
            }
            (ParserState::LocationParsing(_), ParserState::TranscludeParsing) => {
                panic!("Must start parsing a link first")
            }
            (ParserState::LocationParsing(_), ParserState::TranscludeEnd) => {
                panic!("Must start parsing a link first")
            }
            // event disregarded here
            (ParserState::LinkEnd, ParserState::Accept) => {
                self.state = ParserState::Accept;
                Event::Text("]".into())
            }
            (ParserState::LinkEnd, ParserState::LinkStart) => {
                panic!("must start parsing a link before ending")
            }
            (ParserState::LinkEnd, ParserState::LocationParsing(_)) => {
                // Event disregarded here
                {
                    {
                        self.state = message;
                        Event::Text("".into())
                    }
                }
            }
            (ParserState::LinkEnd, ParserState::LinkEnd) => {
                // Event disregarded here
                let location: &str;
                let text: &str;
                let link_location = self.link_location.clone(); // TODO: figure out why I have immutable borrow issues when I don't clone this.
                if link_location.borrow().contains('|') {
                    let split_vals: Vec<&str> = link_location.borrow_mut().split('|').collect();
                    assert!(
                        split_vals.len() < 3,
                        "Malformed wiki link: {} ---> {:?}",
                        self.link_location.borrow(),
                        split_vals
                    );
                    location = split_vals[1];
                    text = split_vals[0];
                } else {
                    location = &link_location.borrow();
                    text = &link_location.borrow();
                }
                let href = format_links(location);
                self.link_location.borrow_mut().clear();
                self.outlinks.borrow_mut().push(location.to_string());
                {
                    self.state = ParserState::Accept;
                    Event::Html(format!(r#"<a href="{}">{}</a>"#, href, text).into())
                }
            }
            (ParserState::LinkEnd, ParserState::TranscludeStart) => {
                panic!("Must start parsing a link first")
            }
            (ParserState::LinkEnd, ParserState::TranscludeParsing) => {
                panic!("Must start parsing a link first")
            }
            (ParserState::LinkEnd, ParserState::TranscludeEnd) => {
                panic!("Must start parsing a link first")
            }
            (ParserState::TranscludeStart, ParserState::Accept) => {
                self.state = message;
                Event::Text("".into())
            }
            (ParserState::TranscludeStart, ParserState::LinkStart) => {
                // [{skjfkj}]
                self.state = ParserState::Accept;
                Event::Text("".into())
            }
            (ParserState::TranscludeStart, ParserState::LocationParsing(_)) => {
                // [[{some weird text}|https://example.com]]
                self.state = ParserState::LocationParsing("".into());
                Event::Text("".into())
            }
            (ParserState::TranscludeStart, ParserState::LinkEnd) => {
                // ]}
                panic!("Must close existing link")
            }
            (ParserState::TranscludeStart, ParserState::TranscludeStart) => {
                self.state = ParserState::TranscludeStart;
                Event::Text("".into())
            }
            (ParserState::TranscludeStart, ParserState::TranscludeParsing) => {
                panic!("Must finish current transclusion first")
            }
            (ParserState::TranscludeStart, ParserState::TranscludeEnd) => {
                panic!("Must finish current transclusion first")
            }
            (ParserState::TranscludeParsing, ParserState::Accept) => {
                panic!("Must start transclusion first")
            }
            (ParserState::TranscludeParsing, ParserState::LinkStart) => {
                panic!("Must start transclusion first")
            }
            (ParserState::TranscludeParsing, ParserState::LocationParsing(_)) => {
                panic!("Must start transclusion first")
            }
            (ParserState::TranscludeParsing, ParserState::LinkEnd) => {
                panic!("Must start transclusion first")
            }
            (ParserState::TranscludeParsing, ParserState::TranscludeStart) => {
                self.state = message;
                Event::Text("".into())
            }
            (ParserState::TranscludeParsing, ParserState::TranscludeParsing) => {
                self.state = ParserState::TranscludeParsing;
                Event::Text("".into())
            }
            (ParserState::TranscludeParsing, ParserState::TranscludeEnd) => {
                panic!("Must start transclusion first")
            }
            (ParserState::TranscludeEnd, ParserState::Accept) => {
                panic!("Must start transclusion first")
            }
            (ParserState::TranscludeEnd, ParserState::LinkStart) => {
                // [}
                {
                    self.state = ParserState::Accept;
                    Event::Text("".into())
                }
            }
            (ParserState::TranscludeEnd, ParserState::LocationParsing(_)) => {
                // [[}my note
                {
                    self.state = ParserState::LocationParsing("".into());
                    Event::Text("".into())
                }
            }

            (ParserState::TranscludeEnd, ParserState::LinkEnd) => panic!("must close the link!"), // ]} [[my note kjsdfkj kdjf ]}
            (ParserState::TranscludeEnd, ParserState::TranscludeStart) => {
                // {}
                {
                    self.state = ParserState::Accept;
                    Event::Text("".into())
                }
            }
            (ParserState::TranscludeEnd, ParserState::TranscludeParsing) => {
                self.state = message;
                Event::Text("".into())
            }
            (ParserState::TranscludeEnd, ParserState::TranscludeEnd) => {
                self.state = message;
                Event::Text("".into())
            }
        }
    }
}

pub fn to_html(md: &str) -> Html {
    let outlinks: Vec<String> = Vec::new();
    let parser_machine = ParserMachine {
        outlinks: RefCell::new(outlinks),
        link_location: RefCell::new(String::new()),
        state: ParserState::Accept,
    };
    let parser = Parser::new_ext(md, Options::all()).map(|event| {
        match event {
            Event::Text(text) => match &*text {
                "[" => parser_machine.send(ParserState::LinkEnd),
                _ => match parser_machine.state {
                    ParserState::LocationParsing(_) => {
                        parser_machine.send(ParserState::LocationParsing(text))
                    }
                    ParserState::LinkEnd => {
                        parser_machine.send(ParserState::Accept);
                        Event::Text(format!("]{}", text).into())
                    }
                    ParserState::LinkStart => {
                        parser_machine.send(ParserState::Accept);
                        Event::Text(format!("[{}", text).into())
                    }
                    _ => {
                        // TODO: custom url schemas?
                        if text.starts_with("http") {
                            return Event::Html(
                                format!(r#"<a href="{}">{}</a>"#, text, text).into(),
                            );
                        }
                        Event::Text(text)
                    }
                },
            },
            _ => event,
        }
    });
    let mut output = String::new();
    html::push_html(&mut output, parser);
    Html {
        body: output,
        outlinks: *parser_machine.outlinks.borrow(),
    }
}

pub fn format_links(link: &str) -> String {
    let proto_prefixes = link.split(':').collect::<Vec<&str>>();
    match proto_prefixes[0] {
        "http" | "https" => link.to_string(),
        "files" => {
            format!("/files/{}", encode(link.strip_prefix("files:").unwrap()))
        }
        _ => format!("/{}", encode(link)), // HACK: deal with warp decoding this later
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_links_properly() {
        let http_link = "https://example.com";
        assert_eq!(String::from("https://example.com"), format_links(http_link));
        let wiki_page = "My Cool Page";
        assert_eq!(String::from("/My%20Cool%20Page"), format_links(wiki_page));
    }
    #[test]
    fn state_transitions_work() {
        let outlinks: Vec<String> = Vec::new();
        let parser_machine = ParserMachine {
            link_location: RefCell::new(String::new()),
            outlinks: RefCell::new(outlinks),
            state: ParserState::Accept,
        };

        let event = parser_machine.send(ParserState::LinkStart);

        assert_eq!(event, Event::Text("".into()));
    }
    #[test]
    #[ignore]
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
