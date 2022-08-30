use std::fmt::Write as _;
use tendril::StrTendril;
use urlencoding::encode;

use super::block::BlockElement;

impl BlockElement {
    pub fn collapse_to(&self, target: &mut String) {
        match self {
            BlockElement::Heading(content) => {
                write!(target, "<h2>").unwrap();
                for part in content {
                    part.collapse_to(target);
                }
                write!(target, "</h2>").unwrap();
            }
            BlockElement::PageLink(content) => {
                let aliases = content.split('|').collect::<Vec<&str>>();
                if aliases.len() > 1 {
                    write!(
                        target,
                        r#"<a href="{}">{}</a>"#,
                        format_links(aliases[1]),
                        aliases[0]
                    )
                    .unwrap();
                } else {
                    write!(
                        target,
                        r#"<a href="{}">{}</a>"#,
                        format_links(aliases[0]),
                        aliases[0]
                    )
                    .unwrap();
                }
            }
            BlockElement::Quote(content) => {
                write!(target, "<blockquote>").unwrap();
                for part in content {
                    part.collapse_to(target);
                }
                write!(target, "</blockquote>").unwrap();
            }
            BlockElement::EmptySpace(content) | BlockElement::Text(content) => {
                write_to_string(target, content.into());
            }
            BlockElement::HyperLink(content) => {
                if content.contains("youtube.com") || content.contains("youtu.be") {
                    write_to_string(target, transform_youtube_url(content));
                }
                if content.contains("codesandbox.io") {
                    write_to_string(target, transform_cs_url(content));
                }
                if content.contains("codepen.io") {
                    write_to_string(target, transform_cp_url(content));
                }
                if content.ends_with(".mp3")
                    || content.ends_with(".ogg")
                    || content.ends_with(".flac")
                {
                    write_to_string(target, transform_audio_url(content));
                }
                if content.contains("vimeo.com") {
                    write_to_string(target, transform_vimeo_url(content));
                }
                if content.contains("spotify.com") {
                    write_to_string(target, transform_spotify_url(content));
                }
                write_to_string(target, format!(r#"<a href="{}">{}</a>"#, content, content));
            }
        }
    }
}

fn write_to_string(target: &mut String, incl: String) {
    write!(target, "{}", incl).unwrap();
}

pub fn transform_audio_url(text: &StrTendril) -> String {
    format!(r#"<audio src="{}" controls></audio>"#, text)
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

pub(crate) fn transform_cs_url(link: &StrTendril) -> String {
    let link = link.replace(".io/s", ".io/embed");
    format!(r#"{} src="{}"></iframe>"#, CS_FMT_STRING, link)
}

pub(crate) fn transform_cp_url(text: &StrTendril) -> String {
    if !text.contains("/embed/") {
        let link = text.replace("/pen/", "/embed/");
        return format!(r#"{} src="{}"></iframe>"#, CP_FMT_STRING, link);
    }
    format!(r#"{} src="{}"></iframe>"#, CP_FMT_STRING, text)
}

pub(crate) fn transform_spotify_url(text: &StrTendril) -> String {
    if !text.contains(".com/embed") {
        let link = text.replace(".com/track", ".com/embed/track");
        return format!(r#"{} src="{}"></iframe>"#, MEDIA_FMT_STRING, link);
    }
    format!(r#"{} src="{}"></iframe>"#, MEDIA_FMT_STRING, text)
}

pub(crate) fn transform_youtube_url(link: &StrTendril) -> String {
    if link.contains("watch?v=") {
        let mut formatted_link = link.replace("watch?v=", "embed/");
        let extra_params_start = formatted_link.find('&');
        if extra_params_start.is_some() {
            formatted_link = formatted_link.replacen('&', "?", 1);
        }
        return format_yt_url(formatted_link);
    }
    // Case of video linked with timestamp
    if !link.contains("embed") && link.contains(".be") {
        let formatted_link = link.replace(".be/", "be.com/embed/");
        return format_yt_url(formatted_link);
    }
    format_yt_url(link.to_string())
}

pub(crate) fn format_yt_url(src: String) -> String {
    format!(r#"{} src="{}"></iframe>"#, MEDIA_FMT_STRING, src)
}

pub(crate) fn transform_vimeo_url(text: &StrTendril) -> String {
    if !text.contains("player.vimeo.com") {
        let link = text.replace("vimeo.com", "player.vimeo.com/video");
        return format!(r#"{} src="{}"></iframe>"#, MEDIA_FMT_STRING, link);
    }
    format!(r#"{} src="{}"></iframe>"#, MEDIA_FMT_STRING, text)
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
    fn transforms_youtube_urls_to_embedable() {
        let link = StrTendril::from_slice("https://youtube.com/watch?v=giEnkiRHJ9Y");
        let final_string = r#"<iframe title="Video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen src="https://youtube.com/embed/giEnkiRHJ9Y"></iframe>"#;
        assert!(StrTendril::from_slice(final_string).to_string() == transform_youtube_url(&link));
    }

    #[test]
    fn transforms_vimeo_urls_to_embedable() {
        let link = StrTendril::from_slice("https://vimeo.com/665036978#t=20s");
        let final_string = r#"<iframe title="Video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen src="https://player.vimeo.com/video/665036978#t=20s"></iframe>"#;
        assert!(StrTendril::from_slice(final_string).to_string() == transform_vimeo_url(&link));
    }
    #[test]
    fn transforms_spotify_urls_to_embedable() {
        let link = StrTendril::from_slice(
            "https://open.spotify.com/track/3YD9EehnGOf88rGSZFrnHg?si=8c669e6880f54c88",
        );
        let final_string = r#"<iframe title="Video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen src="https://open.spotify.com/embed/track/3YD9EehnGOf88rGSZFrnHg?si=8c669e6880f54c88"></iframe>"#;
        assert!(StrTendril::from_slice(final_string).to_string() == transform_spotify_url(&link));
    }

    #[test]
    fn transforms_codepen_urls_to_embedable() {
        let link = StrTendril::from_slice("https://codepen.io/P1N2O/pen/pyBNzX");
        let final_string = r#"<iframe frameborder="0" title="CodePen" scrolling="no" allowtransparency="true" allowfullscreen="true" loading="lazy" src="https://codepen.io/P1N2O/embed/pyBNzX"></iframe>"#;
        assert!(StrTendril::from_slice(final_string).to_string() == transform_cp_url(&link));
    }
}
