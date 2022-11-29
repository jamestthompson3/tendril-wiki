use std::{collections::HashMap, time::Duration};

use persistance::fs::write;
use regex::Regex;
use tokio::time::timeout;
use urlencoding::encode;
use wikitext::{processors::sanitize_html, PatchData};

use crate::{archive::extract, messages::Message, Queue, QueueHandle};

pub struct BookmarkRunner {}

impl BookmarkRunner {
    async fn new_from_url(url: String, tags: Vec<String>) -> Result<(String, PatchData), ()> {
        let mut metadata = HashMap::new();
        metadata.insert(String::from("url"), url.clone());
        if let Ok(product) = tokio::task::spawn_blocking(move || extract(url)).await {
            metadata.insert("content-type".into(), "html".into());
            let title = normalize_title(&product.title);
            let patch = PatchData {
                body: sanitize_html(&product.content),
                tags,
                title,
                old_title: String::with_capacity(0),
                metadata,
            };
            Ok((product.text, patch))
        } else {
            eprintln!("Error in archiving url");
            Err(())
        }
    }

    pub async fn create(form_body: HashMap<String, String>, queue: QueueHandle) -> String {
        let url = form_body.get("url").unwrap();
        let mut tags = form_body
            .get("tags")
            .unwrap()
            .split(',')
            .map(|s| s.to_owned())
            .collect::<Vec<String>>();
        tags.push(String::from("bookmark"));
        if let Ok((archive_body, patch)) = timeout(
            Duration::from_millis(2000),
            BookmarkRunner::new_from_url(url.clone(), tags.clone()),
        )
        .await
        .unwrap()
        {
            match write(&patch).await {
                Ok(()) => {
                    queue
                        .push(Message::Patch {
                            patch: patch.clone(),
                        })
                        .await
                        .unwrap();

                    queue
                        .push(Message::ArchiveBody {
                            title: patch.title.clone(),
                            body: archive_body,
                        })
                        .await
                        .unwrap();
                    return format!("/{}", encode(&patch.title));
                }
                Err(e) => {
                    eprintln!("  {}\n", e);
                    return format!("/error?msg={}", encode(&format!("{:?}", e)));
                }
            }
        } else {
            queue
                .push(Message::NewFromUrl {
                    url: url.to_string(),
                    tags,
                })
                .await
                .unwrap();
        }
        String::from("/bookmark")
    }
}

lazy_static! {
    static ref TITLE_RGX: Regex = Regex::new(r"\?|\\|/|\||:|;|>|<|,|\.|\n|\$|&").unwrap();
}

fn normalize_title(title: &str) -> String {
    let normalized_title = TITLE_RGX.replace_all(title, "");
    // OS file systems don't like really long names, so we can split off bits from the page
    // title if it is too long.
    let mut title = normalized_title.to_string();
    if normalized_title.len() > 50 {
        let (shortened_title, rest) = normalized_title.split_at(50);
        // If it's really long, then we append ellipses. If not, we can just keep the
        // original title.
        if rest.len() > 10 {
            title = format!("{}...", shortened_title.trim());
        }
    }
    title
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_wiki_title() {
        let mut test_title = "testing: a neat thing";
        let result = normalize_title(test_title);
        assert_ne!(String::from(test_title), result);
        assert_eq!(String::from("testing a neat thing"), result);
        test_title =
            "lots of characters. A really long title. Maybe with some / and \\ and -- chars";
        let result = normalize_title(test_title);
        assert_ne!(String::from(test_title), result);
        assert_eq!(
            String::from("lots of characters A really long title Maybe with..."),
            result
        );
    }
}
