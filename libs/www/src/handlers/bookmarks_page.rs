use std::{collections::HashMap, time::Duration};

use persistance::fs::write;
use regex::Regex;
use render::{bookmark_page::BookmarkAddPage, Render, sanitize_html};
use task_runners::{
    archive::extract,
    messages::{Message, PatchData},
    Queue,
};
use tokio::time::timeout;
use urlencoding::encode;
use warp::{filters::BoxedFilter, hyper::Uri, Filter, Reply};

use super::{
    filters::{with_auth, with_queue},
    QueueHandle, MAX_BODY_SIZE,
};

lazy_static! {
    static ref TITLE_RGX: Regex = Regex::new(r"\?|\\|/|\||:|;|>|<|,|\.|\n|\$|&").unwrap();
}

pub struct BookmarkPageRouter {
    queue: QueueHandle,
}

struct Runner {}

impl Runner {
    async fn render() -> String {
        let ctx = BookmarkAddPage {};
        ctx.render().await
    }

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

    async fn create(form_body: HashMap<String, String>, queue: QueueHandle) -> Uri {
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
            Runner::new_from_url(url.clone(), tags.clone()),
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
                    let encoded_title = encode(&patch.title);
                    let redirect_url = &format!("/{}", encoded_title);

                    return redirect_url.parse::<Uri>().unwrap();
                }
                Err(e) => {
                    eprintln!("  {}\n", e);
                    let redir_url = format!("/error?msg={}", encode(&format!("{:?}", e)));
                    return redir_url.parse::<Uri>().unwrap();
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
        let redir_uri = "/bookmark";
        redir_uri.parse::<Uri>().unwrap()
    }
}

impl BookmarkPageRouter {
    pub fn new(queue: QueueHandle) -> Self {
        Self { queue }
    }
    pub fn routes(&self) -> BoxedFilter<(impl Reply,)> {
        warp::any()
            .and(warp::path("new_bookmark"))
            .and(self.get().or(self.post()))
            .boxed()
    }
    fn get(&self) -> BoxedFilter<(impl Reply,)> {
        warp::get()
            .and(with_auth())
            .then(|| async {
                let template = Runner::render().await;
                warp::reply::html(template)
            })
            .boxed()
    }
    fn post(&self) -> BoxedFilter<(impl Reply,)> {
        warp::post()
            .and(with_auth())
            .and(warp::body::content_length_limit(MAX_BODY_SIZE).and(warp::body::form()))
            .and(with_queue(self.queue.to_owned()))
            .then(|form: HashMap<String, String>, queue: QueueHandle| async {
                let response = Runner::create(form, queue).await;
                warp::redirect(response)
            })
            .boxed()
    }
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
        test_title = "lots of characters. A really long title. Maybe with some / and \\ and -- chars";
        let result = normalize_title(test_title);
        assert_ne!(String::from(test_title), result);
        assert_eq!(String::from("lots of characters A really long title Maybe with..."), result);
    }
}
