use pulldown_cmark::CowStr;
use regex::Regex;
// use pulldown_cmark::{html, CowStr, Event, Options, Parser};
use urlencoding::encode;

lazy_static! {
    pub(crate) static ref WIKI_LINK_END: Regex =
        Regex::new(r#"(.*)(\]\])([[:punct:]]?$)"#).unwrap();
}

#[derive(Clone, PartialEq, Debug)]
enum ParserState {
    LocationParsing(String),
    Accept,
}

pub struct Html {
    pub outlinks: Vec<String>,
    pub body: String,
}

// TODO maybe this can take ownership and mutate the string.
pub fn to_html(md: &str) -> Html {
    let mut final_rendered_output: Vec<String> = Vec::new();

    let mut outlinks = Vec::new();
    let lines = md.split('\n');
    for line in lines {
        if line.starts_with('#') {
            final_rendered_output.push(format_text_block(format!(
                "<h2>{}</h2>",
                line.strip_prefix('#').unwrap().trim()
            )));
            continue;
        }
        println!("{}", line);
        let mut parsed_line = Vec::new();
        let mut parser_state = ParserState::Accept;
        for word in line.split(' ') {
            if word.starts_with("[[") && WIKI_LINK_END.is_match(word) {
                let aliases = word
                    .strip_prefix("[[")
                    .unwrap()
                    .split('|')
                    .collect::<Vec<&str>>();
                let caps = WIKI_LINK_END.captures(word).unwrap();
                let wiki_link_end = caps.get(2).unwrap().as_str();
                let wiki_link_punct = match caps.get(3) {
                    Some(punct) => punct.as_str(),
                    None => "",
                };
                if aliases.len() > 1 {
                    if !aliases[1].starts_with("http") {
                        outlinks.push(aliases[1].to_string());
                    }
                    construct_wiki_link(&mut parsed_line, (&format_links(aliases[1]), aliases[0]));
                } else {
                    if !aliases[0].starts_with("http") {
                        outlinks.push(aliases[0].to_string());
                    }
                    construct_wiki_link(&mut parsed_line, (&format_links(aliases[0]), aliases[0]));
                }
                continue;
            }
            if word.starts_with("[[") {
                parser_state =
                    ParserState::LocationParsing(word.strip_prefix("[[").unwrap().into());
                continue;
            }
            if WIKI_LINK_END.is_match(word) {
                println!("{} --> {:?}", word, parser_state);
                match parser_state {
                    ParserState::Accept => {
                        parsed_line.push(word.into());
                    }
                    ParserState::LocationParsing(prev_word) => {
                        let caps = WIKI_LINK_END.captures(word).unwrap();
                        let wiki_link_end = caps.get(2).unwrap().as_str();
                        let wiki_link_punct = match caps.get(3) {
                            Some(punct) => punct.as_str(),
                            None => "",
                        };
                        if prev_word.ends_with('|') {
                            if !word.starts_with("http") {
                                outlinks.push(prev_word.clone());
                            }
                            let trimmed_word = prev_word.strip_suffix('|').unwrap();
                            construct_wiki_link(
                                &mut parsed_line,
                                (&format_links(trimmed_word), word),
                            );
                        } else {
                            let caps = WIKI_LINK_END.captures(word).unwrap();
                            let wiki_link_end = caps.get(2).unwrap().as_str();
                            let wiki_link_punct = match caps.get(3) {
                                Some(punct) => punct.as_str(),
                                None => "",
                            };
                            let second_half =
                                word.replace(wiki_link_end, "").replace(wiki_link_punct, "");
                            let constructed_word = format!("{} {}", prev_word, second_half);
                            println!("  {}", constructed_word);
                            if !word.starts_with("http") {
                                outlinks.push(constructed_word.clone());
                            }
                            construct_wiki_link(
                                &mut parsed_line,
                                (
                                    &format_links(&constructed_word),
                                    &format!("{} {}", prev_word, word),
                                ),
                            );
                        }
                        parser_state = ParserState::Accept;
                    }
                }
                continue;
            }
            parsed_line.push(word.into());
        }
        final_rendered_output.push(format_text_block(parsed_line.join(" ")));

        // TODO: custom url schemas?
        // if text.starts_with("http") {
        //     if text.contains("youtube.com") || text.contains("youtu.be") {
        //         return Event::Html(transform_youtube_url(text));
        //     }
        //     if text.contains("codesandbox.io") {
        //         return Event::Html(transform_cs_url(text));
        //     }
        //     if text.contains("codepen.io") {
        //         return Event::Html(transform_cp_url(text));
        //     }
        //     if text.ends_with(".mp3")
        //         || text.ends_with(".ogg")
        //         || text.ends_with(".flac")
        //     {
        //         return Event::Html(transform_audio_url(text));
        //     }
        //     if text.contains("vimeo.com") {
        //         return Event::Html(transform_vimeo_url(text));
        //     }
        //     if text.contains("spotify.com") {
        //         return Event::Html(transform_spotify_url(text));
        //     }

        //     return Event::Html(format!(r#"<a href="{}">{}</a>"#, text, text).into());
        // }
        // Event::Text(text)
        // }
        // },
        // },
        // }
    }
    Html {
        body: final_rendered_output.join(""),
        outlinks,
    }
}

fn construct_wiki_link(string_collection: &mut Vec<String>, parts: (&str, &str)) {
    let caps = WIKI_LINK_END.captures(parts.1).unwrap();
    let wiki_link_punct = match caps.get(3) {
        Some(punct) => punct.as_str(),
        None => "",
    };
    let formatted_link_content = parts.1.replace("]]", "").replace(wiki_link_punct, "");
    string_collection.push(format!(
        "<a href=\"{}\">{}</a>{}",
        parts.0, formatted_link_content, wiki_link_punct
    ));
}

fn format_text_block(inner: String) -> String {
    format!("<div class=\"text-block\">{}</div>", inner)
}

fn transform_audio_url(text: CowStr) -> CowStr {
    format!(r#"<audio src="{}" controls></audio>"#, text).into()
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

const MEDIA_FMT_STRING: &str = r#"<iframe title="Video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen"#;
const CS_FMT_STRING: &str = r#"<iframe frameborder="0" title="Code Sandbox" allow="accelerometer; ambient-light-sensor;
    camera; encrypted-media; geolocation; gyroscope; hid; microphone; midi; payment; usb; vr;
    xr-spatial-tracking" sandbox="allow-forms allow-modals allow-popups allow-presentation
    allow-same-origin allow-scripts""#;
const CP_FMT_STRING: &str = r#"<iframe frameborder="0" title="CodePen" scrolling="no" allowtransparency="true" allowfullscreen="true" loading="lazy""#;

fn transform_cs_url(link: CowStr) -> CowStr {
    let link = link.replace(".io/s", ".io/embed");
    format!(r#"{} src="{}"></iframe>"#, CS_FMT_STRING, link).into()
}
fn transform_cp_url(text: CowStr) -> CowStr {
    if !text.contains("/embed/") {
        let link = text.replace("/pen/", "/embed/");
        return format!(r#"{} src="{}"></iframe>"#, CP_FMT_STRING, link).into();
    }
    format!(r#"{} src="{}"></iframe>"#, CP_FMT_STRING, text).into()
}

fn transform_spotify_url(text: CowStr) -> CowStr {
    if !text.contains(".com/embed") {
        let link = text.replace(".com/track", ".com/embed/track");
        return format!(r#"{} src="{}"></iframe>"#, MEDIA_FMT_STRING, link).into();
    }
    format!(r#"{} src="{}"></iframe>"#, MEDIA_FMT_STRING, text).into()
}

fn transform_youtube_url(link: CowStr) -> CowStr {
    if link.contains("watch?v=") {
        let mut formatted_link = link.replace("watch?v=", "embed/");
        let extra_params_start = formatted_link.find('&');
        if extra_params_start.is_some() {
            formatted_link = formatted_link.replacen('&', "?", 1);
        }
        return format_yt_url(formatted_link).into();
    }
    // Case of video linked with timestamp
    if !link.contains("embed") && link.contains(".be") {
        let formatted_link = link.replace(".be/", "be.com/embed/");
        return format_yt_url(formatted_link).into();
    }
    format_yt_url(link.to_string()).into()
}

fn format_yt_url(src: String) -> String {
    format!(r#"{} src="{}"></iframe>"#, MEDIA_FMT_STRING, src)
}

fn transform_vimeo_url(text: CowStr) -> CowStr {
    if !text.contains("player.vimeo.com") {
        let link = text.replace("vimeo.com", "player.vimeo.com/video");
        return format!(r#"{} src="{}"></iframe>"#, MEDIA_FMT_STRING, link).into();
    }
    format!(r#"{} src="{}"></iframe>"#, MEDIA_FMT_STRING, text).into()
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
    fn parses_md_to_html_with_wikilinks() {
        let test_string = "[[Some Page]]";
        let test_html = Html {
            outlinks: vec!["Some Page".into()],
            body: "<div class=\"text-block\"><a href=\"/Some%20Page\">Some Page</a></div>".into(),
        };
        let parsed = to_html(test_string);
        assert_eq!(parsed.outlinks, test_html.outlinks);
        assert_eq!(parsed.body, test_html.body);

        let test_string = "# Title\n[[Some Page]]. Another thing\n * Hi\n * List\n * Output";
        let test_html = Html {
            outlinks: vec!["Some Page".into()],
            body: "<div class=\"text-block\"><h2>Title</h2></div><div class=\"text-block\"><a href=\"/Some%20Page\">Some Page</a>. Another thing</div><div class=\"text-block\"> * Hi</div><div class=\"text-block\"> * List</div><div class=\"text-block\"> * Output</div>".into()
        };
        let parsed = to_html(test_string);
        assert_eq!(parsed.outlinks, test_html.outlinks);
        assert_eq!(parsed.body, test_html.body);
    }

    #[test]
    fn transforms_youtube_urls_to_embedable() {
        let link = CowStr::from("https://youtube.com/watch?v=giEnkiRHJ9Y");
        let final_string = r#"<iframe title="Video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen src="https://youtube.com/embed/giEnkiRHJ9Y"></iframe>"#;
        assert!(CowStr::from(final_string).to_string() == transform_youtube_url(link).to_string());
    }

    #[test]
    fn transforms_vimeo_urls_to_embedable() {
        let link = CowStr::from("https://vimeo.com/665036978#t=20s");
        let final_string = r#"<iframe title="Video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen src="https://player.vimeo.com/video/665036978#t=20s"></iframe>"#;
        assert!(CowStr::from(final_string).to_string() == transform_vimeo_url(link).to_string());
    }
    #[test]
    fn transforms_spotify_urls_to_embedable() {
        let link = CowStr::from(
            "https://open.spotify.com/track/3YD9EehnGOf88rGSZFrnHg?si=8c669e6880f54c88",
        );
        let final_string = r#"<iframe title="Video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen src="https://open.spotify.com/embed/track/3YD9EehnGOf88rGSZFrnHg?si=8c669e6880f54c88"></iframe>"#;
        assert!(CowStr::from(final_string).to_string() == transform_spotify_url(link).to_string());
    }

    #[test]
    fn transforms_codepen_urls_to_embedable() {
        let link = CowStr::from("https://codepen.io/P1N2O/pen/pyBNzX");
        let final_string = r#"<iframe frameborder="0" title="CodePen" scrolling="no" allowtransparency="true" allowfullscreen="true" loading="lazy" src="https://codepen.io/P1N2O/embed/pyBNzX"></iframe>"#;
        assert!(CowStr::from(final_string).to_string() == transform_cp_url(link).to_string());
    }
}
