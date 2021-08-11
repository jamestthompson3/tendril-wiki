use pulldown_cmark::{html, Event, Options, Parser};

use super::machine::{ParserMachine, ParserMachineContext, ParserState};

pub struct Html {
    pub outlinks: Vec<String>,
    pub body: String,
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

#[cfg(test)]
mod tests {
    use super::*;

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
