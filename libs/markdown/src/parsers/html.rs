use pulldown_cmark::{html, Event, Options, Parser};
use urlencoding::encode;

#[derive(Copy, Clone, PartialEq, Debug)]
enum ParserState {
    LinkStart,
    LocationParsing,
    LinkEnd,
    Accept,
}

pub struct Html {
    pub outlinks: Vec<String>,
    pub body: String,
}

#[derive(Copy, Clone)]
struct ParserMachine {
    state: ParserState,
}

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
    pub fn send(&mut self, state: ParserState) {
        self.state = state;
    }
}

pub fn to_html(md: &str) -> Html {
    // TODO maybe don't allocate...
    let mut wiki_link_location = String::new();

    let mut parser_machine = ParserMachine::new();
    let mut outlinks = Vec::new();
    let parser = Parser::new_ext(&md, Options::all()).map(|event| match event {
        Event::Text(text) => match &*text {
            "[" => match parser_machine.current_state() {
                ParserState::Accept => {
                    parser_machine.send(ParserState::LinkStart);
                    Event::Text("".into())
                }
                ParserState::LinkStart => {
                    parser_machine.send(ParserState::LocationParsing);
                    Event::Text("".into())
                }
                _ => {
                    println!("{}\n\n{:?}", md, parser_machine.current_state());
                    panic!("Impossible state reached for `[`");
                }
            },
            "]" => match parser_machine.current_state() {
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
                    parser_machine.send(ParserState::Accept);
                    Event::Html(format!(r#"<a href="{}">{}</a>"#, link_location, text).into())
                }
                ParserState::LocationParsing => {
                    parser_machine.send(ParserState::LinkEnd);
                    Event::Text("".into())
                }
                ParserState::Accept => Event::Text(text),
                _ => {
                    println!("{:?}", parser_machine.current_state());
                    panic!("Impossible statereached for `]`");
                }
            },
            _ => match parser_machine.current_state() {
                ParserState::LocationParsing => {
                    wiki_link_location.push_str(&text);
                    Event::Text("".into())
                }
                ParserState::LinkEnd => {
                    parser_machine.send(ParserState::LocationParsing);
                    Event::Text(format!("]{}", text).into())
                }
                ParserState::LinkStart => {
                    parser_machine.send(ParserState::Accept);
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
