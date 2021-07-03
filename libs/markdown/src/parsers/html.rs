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

#[derive(Clone, Debug)]
struct ParserMachineContext {
    link_location: String,
    outlinks: Vec<String>,
}

#[derive(Clone, Debug)]
struct ParserMachine<'a> {
    state: ParserState<'a>,
    context: ParserMachineContext,
}

impl<'a> ParserMachine<'a> {
    fn send(mut self, message: ParserState<'a>) -> (ParserMachine, Event) {
        match (&message, &self.state) {
            (ParserState::Accept, ParserState::Accept) => (
                ParserMachine {
                    state: message,
                    context: self.context,
                },
                Event::Text("".into()),
            ),
            (ParserState::Accept, ParserState::LinkEnd) => (
                ParserMachine {
                    state: message,
                    context: self.context,
                },
                Event::Text("".into()),
            ),

            (ParserState::Accept, ParserState::LinkStart) => {
                // [checkout thing]
                (
                    ParserMachine {
                        state: message,
                        context: self.context,
                    },
                    Event::Text("".into()),
                )
            }
            (ParserState::Accept, ParserState::LocationParsing(_)) => panic!("Unclosed link"), // [[kajdf akdj
            (ParserState::Accept, ParserState::TranscludeStart) => {
                // {some brackets}
                (
                    ParserMachine {
                        state: message,
                        context: self.context,
                    },
                    Event::Text("".into()),
                )
            }
            (ParserState::Accept, ParserState::TranscludeParsing) => {
                panic!("Unclosed transclusion")
            }
            (ParserState::Accept, ParserState::TranscludeEnd) => (
                ParserMachine {
                    state: message,
                    context: self.context,
                },
                Event::Text("".into()),
            ),

            (ParserState::LinkStart, ParserState::Accept) => (
                ParserMachine {
                    state: message,
                    context: self.context,
                },
                Event::Text("".into()),
            ),

            (ParserState::LinkStart, ParserState::LinkStart) => (
                ParserMachine {
                    state: ParserState::LocationParsing("".into()),
                    context: self.context,
                },
                Event::Text("".into()),
            ),

            (ParserState::LinkStart, ParserState::LocationParsing(_)) => {
                panic!("Link parsing has already started")
            }

            (ParserState::LinkStart, ParserState::LinkEnd) => {
                panic!("Can't start link immediately from ending")
            }

            (ParserState::LinkStart, ParserState::TranscludeStart) => {
                // {[some brackets]}
                (
                    ParserMachine {
                        state: message,
                        context: self.context,
                    },
                    Event::Text("".into()),
                )
            }
            (ParserState::LinkStart, ParserState::TranscludeParsing) => (
                ParserMachine {
                    state: message,
                    context: self.context,
                },
                Event::Text("".into()),
            ),

            (ParserState::LinkStart, ParserState::TranscludeEnd) => {
                panic!("Can't start link immediately from transclude ending")
            }
            (ParserState::LocationParsing(_), ParserState::Accept) => {
                panic!("must start parsing a link first")
            }
            (ParserState::LocationParsing(text), ParserState::LinkStart) => (
                ParserMachine {
                    state: message.to_owned(),
                    context: self.context,
                },
                Event::Text(text.to_owned()),
            ),
            (ParserState::LocationParsing(text), ParserState::LocationParsing(_)) => {
                self.context.link_location.push_str(text);
                {
                    (
                        ParserMachine {
                            state: ParserState::LocationParsing(text.to_owned()),
                            context: self.context,
                        },
                        Event::Text("".into()),
                    )
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
            (ParserState::LinkEnd, ParserState::Accept) => (
                ParserMachine {
                    state: ParserState::Accept,
                    context: self.context,
                },
                Event::Text("]".into()),
            ),
            (ParserState::LinkEnd, ParserState::LinkStart) => {
                panic!("must start parsing a link before ending")
            }
            (ParserState::LinkEnd, ParserState::LocationParsing(_)) => {
                // Event disregarded here
                (
                    ParserMachine {
                        state: message,
                        context: self.context,
                    },
                    Event::Text("".into()),
                )
            }
            (ParserState::LinkEnd, ParserState::LinkEnd) => {
                println!("{:?}", self.context);
                // Event disregarded here
                let location: &str;
                let text: &str;
                let link_location = self.context.link_location.clone(); // TODO: figure out why I have immutable borrow issues when I don't clone this.
                if link_location.contains('|') {
                    let split_vals: Vec<&str> = link_location.split('|').collect();
                    assert!(
                        split_vals.len() < 3,
                        "Malformed wiki link: {} ---> {:?}",
                        self.context.link_location,
                        split_vals
                    );
                    location = split_vals[1];
                    text = split_vals[0];
                } else {
                    location = &link_location;
                    text = &link_location;
                }
                let href = format_links(location);
                self.context.link_location.clear();
                self.context.outlinks.push(location.to_string());
                (
                    ParserMachine {
                        state: ParserState::Accept,
                        context: self.context,
                    },
                    Event::Html(format!(r#"<a href="{}">{}</a>"#, href, text).into()),
                )
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
            (ParserState::TranscludeStart, ParserState::Accept) => (
                ParserMachine {
                    state: message,
                    context: self.context,
                },
                Event::Text("".into()),
            ),
            (ParserState::TranscludeStart, ParserState::LinkStart) => {
                // [{skjfkj}]
                (
                    ParserMachine {
                        state: ParserState::Accept,
                        context: self.context,
                    },
                    Event::Text("".into()),
                )
            }
            (ParserState::TranscludeStart, ParserState::LocationParsing(_)) => {
                // [[{some weird text}|https://example.com]]
                (
                    ParserMachine {
                        state: ParserState::LocationParsing("".into()),
                        context: self.context,
                    },
                    Event::Text("".into()),
                )
            }
            (ParserState::TranscludeStart, ParserState::LinkEnd) => {
                // ]}
                panic!("Must close existing link")
            }
            (ParserState::TranscludeStart, ParserState::TranscludeStart) => (
                ParserMachine {
                    state: ParserState::TranscludeStart,
                    context: self.context,
                },
                Event::Text("".into()),
            ),
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
            (ParserState::TranscludeParsing, ParserState::TranscludeStart) => (
                ParserMachine {
                    state: message,
                    context: self.context,
                },
                Event::Text("".into()),
            ),
            (ParserState::TranscludeParsing, ParserState::TranscludeParsing) => (
                ParserMachine {
                    state: ParserState::TranscludeParsing,
                    context: self.context,
                },
                Event::Text("".into()),
            ),
            (ParserState::TranscludeParsing, ParserState::TranscludeEnd) => {
                panic!("Must start transclusion first")
            }
            (ParserState::TranscludeEnd, ParserState::Accept) => {
                panic!("Must start transclusion first")
            }
            (ParserState::TranscludeEnd, ParserState::LinkStart) => {
                // [}
                (
                    ParserMachine {
                        state: ParserState::Accept,
                        context: self.context,
                    },
                    Event::Text("".into()),
                )
            }
            (ParserState::TranscludeEnd, ParserState::LocationParsing(_)) => {
                // [[}my note
                (
                    ParserMachine {
                        state: ParserState::LocationParsing("".into()),
                        context: self.context,
                    },
                    Event::Text("".into()),
                )
            }

            (ParserState::TranscludeEnd, ParserState::LinkEnd) => panic!("must close the link!"), // ]} [[my note kjsdfkj kdjf ]}
            (ParserState::TranscludeEnd, ParserState::TranscludeStart) => {
                // {}
                (
                    ParserMachine {
                        state: ParserState::Accept,
                        context: self.context,
                    },
                    Event::Text("".into()),
                )
            }
            (ParserState::TranscludeEnd, ParserState::TranscludeParsing) => (
                ParserMachine {
                    state: message,
                    context: self.context,
                },
                Event::Text("".into()),
            ),
            (ParserState::TranscludeEnd, ParserState::TranscludeEnd) => (
                ParserMachine {
                    state: message,
                    context: self.context,
                },
                Event::Text("".into()),
            ),
        }
    }
}

pub fn to_html(md: &str) -> Html {
    let outlinks: Vec<String> = Vec::new();
    let mut parser_machine = ParserMachine {
        context: ParserMachineContext {
            outlinks,
            link_location: String::new(),
        },
        state: ParserState::Accept,
    };
    let parser = Parser::new_ext(md, Options::all());
    let mut final_events: Vec<Event> = Vec::new();
    for event in parser {
        match event {
            Event::Text(text) => match &*text {
                "[" => {
                    let (machine, event) = parser_machine.send(ParserState::LinkStart);
                    parser_machine = machine;
                    final_events.push(event);
                }
                "{" => {
                    let (machine, event) = parser_machine.send(ParserState::TranscludeStart);
                    parser_machine = machine;
                    final_events.push(event);
                }
                "]" => {
                    let (machine, event) = parser_machine.send(ParserState::LinkEnd);
                    parser_machine = machine;
                    final_events.push(event);
                }
                "}" => {
                    let (machine, event) = parser_machine.send(ParserState::TranscludeEnd);
                    parser_machine = machine;
                    final_events.push(event);
                }
                _ => match parser_machine.state {
                    ParserState::LocationParsing(_) => {
                        let (machine, event) =
                            parser_machine.send(ParserState::LocationParsing(text));
                        parser_machine = machine;
                        final_events.push(event);
                    }
                    ParserState::LinkEnd => {
                        let (machine, _) = parser_machine.send(ParserState::Accept);
                        parser_machine = machine;
                        final_events.push(Event::Text(format!("]{}", text).into()));
                    }
                    ParserState::LinkStart => {
                        let (machine, _) = parser_machine.send(ParserState::Accept);
                        parser_machine = machine;
                        final_events.push(Event::Text(format!("[{}", text).into()));
                    }
                    _ => {
                        // TODO: custom url schemas?
                        if text.starts_with("http") {
                            final_events.push(Event::Html(
                                format!(r#"<a href="{}">{}</a>"#, text, text).into(),
                            ));
                        }
                        final_events.push(Event::Text(text));
                    }
                },
            },
            _ => {
                final_events.push(event);
            }
        }
    }

    let mut output = String::new();
    html::push_html(&mut output, final_events.into_iter());
    Html {
        body: output,
        outlinks: parser_machine.context.outlinks,
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
        let mut parser_machine = ParserMachine {
            context: ParserMachineContext {
                link_location: String::new(),
                outlinks,
            },
            state: ParserState::Accept,
        };

        let valid_transitions = vec![
            ParserState::LinkStart,
            ParserState::LinkStart,
            ParserState::LocationParsing("Testing!".into()),
            ParserState::LinkEnd,
            ParserState::Accept,
        ];
        let valid_next_states = vec![
            ParserState::LinkStart,
            ParserState::LocationParsing("".into()),
            ParserState::LocationParsing("Testing!".into()),
            ParserState::LinkEnd,
            ParserState::Accept,
        ];

        let valid_events = vec![
            Event::Text("".into()),
            Event::Text("".into()),
            Event::Text("".into()),
            Event::Text("".into()),
            Event::Text("".into()),
        ];

        for (indx, event) in valid_events.iter().enumerate() {
            let message = &valid_transitions[indx];
            let state = &valid_next_states[indx];
            let (machine, machine_event) = parser_machine.send(message.to_owned());
            parser_machine = machine;
            println!(
                "Indx: {}, Sent: {:?}, Machine State: {:?}, Expected: {:?}",
                indx, message, parser_machine.state, state
            );

            assert_eq!(machine_event, *event);
            assert_eq!(parser_machine.state, *state);
        }
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
