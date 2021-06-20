use pulldown_cmark::{html, Event, Options, Parser};
use urlencoding::encode;

#[derive(Copy, Clone, PartialEq, Debug)]
enum ParserState {
    LinkStart,         // [
    LocationParsing,   // [[
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

#[derive(Copy, Clone)]
struct ParserMachine {
    state: ParserState,
}

// Note A -> Note B -> Note C

impl ParserMachine {
    pub fn new() -> Self {
        ParserMachine {
            state: ParserState::Accept,
        }
    }
    #[inline]
    pub fn current_state(self) -> ParserState {
        self.state
    }
    #[inline]
    pub fn send(&self, current_state: ParserState, message: ParserState) -> (ParserState, Event) {
        match message {
            ParserState::Accept => {
                match current_state {
                    ParserState::Accept => (message, Event::Text("".into())),
                    ParserState::LinkEnd => (message, Event::Text("".into())),
                    ParserState::LinkStart => {
                        // [checkout thing]
                        (message, Event::Text("".into()))
                    }
                    ParserState::LocationParsing => panic!("Unclosed link"), // [[kajdf akdj
                    ParserState::TranscludeStart => {
                        // {some brackets}
                        (message, Event::Text("".into()))
                    }
                    ParserState::TranscludeParsing => panic!("Unclosed transclusion"),
                    ParserState::TranscludeEnd => (message, Event::Text("".into())),
                }
            }

            ParserState::LinkStart => {
                match current_state {
                    ParserState::Accept => (message, Event::Text("".into())),
                    ParserState::LinkStart => {
                        (ParserState::LocationParsing, Event::Text("".into()))
                    }
                    ParserState::LocationParsing => panic!("Link parsing has already started"),
                    ParserState::LinkEnd => panic!("Can't start link immediately from ending"),
                    ParserState::TranscludeStart => {
                        // {[some brackets]}
                        (message, Event::Text("".into()))
                    }
                    ParserState::TranscludeParsing => (message, Event::Text("".into())),
                    ParserState::TranscludeEnd => {
                        panic!("Can't start link immediately from transclude ending")
                    }
                }
            }
            ParserState::LocationParsing => match current_state {
                ParserState::Accept => panic!("must start parsing a link first"),
                ParserState::LinkStart => (message, Event::Text("]".into())),
                ParserState::LocationParsing => {
                    (ParserState::LocationParsing, Event::Text("".into()))
                }
                ParserState::LinkEnd => panic!("Must start parsing a link first"),
                ParserState::TranscludeStart => panic!("Must start parsing a link first"),
                ParserState::TranscludeParsing => panic!("Must start parsing a link first"),
                ParserState::TranscludeEnd => panic!("Must start parsing a link first"),
            },
            ParserState::LinkEnd => {
                match current_state {
                    // event disregarded here
                    ParserState::Accept => (ParserState::Accept, Event::Text("".into())),
                    ParserState::LinkStart => panic!("must start parsing a link before ending"),
                    ParserState::LocationParsing => {
                        // Event disregarded here
                        (message, Event::Text("".into()))
                    }
                    ParserState::LinkEnd => {
                        // Event disregarded here
                        (ParserState::Accept, Event::Text("".into()))
                    }
                    ParserState::TranscludeStart => panic!("Must start parsing a link first"),
                    ParserState::TranscludeParsing => panic!("Must start parsing a link first"),
                    ParserState::TranscludeEnd => panic!("Must start parsing a link first"),
                }
            }
            ParserState::TranscludeStart => {
                match current_state {
                    ParserState::Accept => (message, Event::Text("".into())),
                    ParserState::LinkStart => (ParserState::Accept, Event::Text("".into())), // [{skjfkj}]
                    ParserState::LocationParsing => (current_state, Event::Text("".into())), // [[{some weird text}|https://example.com]]
                    ParserState::LinkEnd => panic!("Must close existing link"),              // ]}
                    ParserState::TranscludeStart => (current_state, Event::Text("".into())),
                    ParserState::TranscludeParsing => {
                        panic!("Must finish current transclusion first")
                    }
                    ParserState::TranscludeEnd => panic!("Must finish current transclusion first"),
                }
            }
            ParserState::TranscludeParsing => match current_state {
                ParserState::Accept => panic!("Must start transclusion first"),
                ParserState::LinkStart => panic!("Must start transclusion first"),
                ParserState::LocationParsing => panic!("Must start transclusion first"),
                ParserState::LinkEnd => panic!("Must start transclusion first"),
                ParserState::TranscludeStart => (message, Event::Text("".into())),
                ParserState::TranscludeParsing => (current_state, Event::Text("".into())),
                ParserState::TranscludeEnd => panic!("Must start transclusion first"),
            },
            ParserState::TranscludeEnd => {
                match current_state {
                    ParserState::Accept => panic!("Must start transclusion first"),
                    ParserState::LinkStart => (ParserState::Accept, Event::Text("".into())), // [}
                    ParserState::LocationParsing => (current_state, Event::Text("".into())), // [[}my note}]]
                    ParserState::LinkEnd => panic!("must close the link!"), // ]} [[my note kjsdfkj kdjf ]}
                    ParserState::TranscludeStart => (ParserState::Accept, Event::Text("".into())), // {}
                    ParserState::TranscludeParsing => (message, Event::Text("".into())),
                    ParserState::TranscludeEnd => (current_state, Event::Text("".into())),
                }
            }
        }
    }
}

pub fn to_html(md: &str) -> Html {
    // TODO maybe don't allocate...
    let mut wiki_link_location = String::new();

    let parser_machine = ParserMachine::new();
    let mut outlinks = Vec::new();
    let mut state: ParserState = ParserState::Accept;
    let parser = Parser::new_ext(&md, Options::all()).map(|event| match event {
        Event::Text(text) => match &*text {
            "[" => {
                let (next, event) = parser_machine.send(state, ParserState::LinkStart);
                state = next;
                event
            }
            "]" => match state {
                ParserState::LinkEnd => {
                    let link_text = wiki_link_location.clone();
                    let location: &str;
                    let text: &str;
                    if link_text.contains('|') {
                        let split_vals: Vec<&str> = link_text.split('|').collect();
                        assert_eq!(
                            split_vals.len() < 3,
                            true,
                            "Malformed wiki link: {} ---> {:?}",
                            link_text,
                            split_vals
                        );
                        location = split_vals[1];
                        text = split_vals[0];
                    } else {
                        location = &link_text;
                        text = &link_text;
                    }
                    let link_location = format_links(location);
                    outlinks.push(location.to_string());
                    wiki_link_location.clear();
                    let (next, _) = parser_machine.send(state, ParserState::Accept);
                    state = next;
                    Event::Html(format!(r#"<a href="{}">{}</a>"#, link_location, text).into())
                }
                ParserState::LocationParsing => {
                    let (next, event) = parser_machine.send(state, ParserState::LinkEnd);
                    state = next;
                    event
                }
                ParserState::Accept => Event::Text(text),
                _ => {
                    println!("{:?}", parser_machine.current_state());
                    panic!("Impossible statereached for `]`");
                }
            },
            _ => match state {
                ParserState::LocationParsing => {
                    wiki_link_location.push_str(&text);
                    Event::Text("".into())
                }
                ParserState::LinkEnd => {
                    let (next, _) = parser_machine.send(state, ParserState::Accept);
                    state = next;
                    Event::Text(format!("]{}", text).into())
                }
                ParserState::LinkStart => {
                    let (next, _) = parser_machine.send(state, ParserState::Accept);
                    state = next;
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
        outlinks,
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
