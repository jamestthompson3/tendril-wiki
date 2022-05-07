use std::{collections::HashMap, sync::Arc};

use build::purge_mru_cache;
use persistance::fs::{create_journal_entry, read, write, ReadPageError};
use render::{link_page::LinkPage, new_page::NewPage, GlobalBacklinks, Render};
use tasks::{
    messages::{Message, PatchData},
    JobQueue, Queue,
};
use urlencoding::{decode, encode};
use warp::{filters::BoxedFilter, hyper::Uri, Filter, Reply};

use crate::RefHubParts;

type QueueHandle = Arc<JobQueue>;

use super::{
    filters::{with_auth, with_links, with_location, with_queue},
    MAX_BODY_SIZE,
};

struct Runner {}

impl Runner {
    pub async fn render_backlink_index(links: GlobalBacklinks) -> String {
        let links = links.lock().await;
        let ctx = LinkPage {
            links: links.to_owned(),
        };
        ctx.render().await
    }

    pub async fn render_file(
        &self,
        path: String,
        reflinks: GlobalBacklinks,
        wiki_location: String,
        query_params: HashMap<String, String>,
    ) -> String {
        self.render_from_path(&wiki_location, path, reflinks, query_params)
            .await
            .unwrap()
    }

    pub async fn render_nested_file(
        mut main_path: String,
        sub_path: String,
        reflinks: GlobalBacklinks,
        wiki_location: String,
    ) -> String {
        // I don't know why warp doesn't decode the sub path here...
        let sub_path_decoded = decode(&sub_path).unwrap();
        main_path.push_str(&format!("/{}", sub_path_decoded));
        let page = read(&wiki_location, main_path.clone(), reflinks).await;
        if page.is_ok() {
            page.unwrap()
        } else {
            println!("Cannot read page: {} due to {:?}", main_path, page.err());
            String::with_capacity(0)
        }
    }

    pub async fn render_from_path(
        &self,
        location: &str,
        path: String,
        links: GlobalBacklinks,
        query_params: HashMap<String, String>,
    ) -> Result<String, ReadPageError> {
        match read(location, path.clone(), links).await {
            Ok(page) => Ok(page),
            Err(ReadPageError::PageNotFoundError) => {
                // TODO: Ideally, I want to redirect, but I'm not sure how to do this with
                // warp's filter system where some branches return HTML, and others redirect...
                let ctx = NewPage {
                    title: Some(urlencoding::decode(&path).unwrap().into_owned()),
                    linkto: query_params.get("linkto"),
                    action_params: None,
                };
                Ok(ctx.render().await)
            }
            _ => Err(ReadPageError::Unknown),
        }
    }
    pub async fn render_new(query_params: HashMap<String, String>) -> String {
        let ctx = NewPage {
            title: None,
            linkto: query_params.get("linkto"),
            action_params: None,
        };
        ctx.render().await
    }

    pub async fn edit(
        form_body: HashMap<String, String>,
        wiki_location: String,
        queue: QueueHandle,
        query_params: HashMap<String, String>,
    ) -> Result<Uri, std::io::Error> {
        let parsed_data = PatchData::from(form_body);
        let redir_uri = if let Some(redirect_addition) = query_params.get("redir_to") {
            format!("/{}/{}", redirect_addition, encode(&parsed_data.title))
        } else {
            format!("/{}", encode(&parsed_data.title))
        };
        if parsed_data
            .tags
            .iter()
            .map(|t| t.to_lowercase())
            .any(|t| t == "bookmark")
        {
            if let Some(url) = parsed_data.metadata.get("url") {
                if parsed_data.old_title != parsed_data.title && !parsed_data.old_title.is_empty() {
                    queue
                        .push(Message::ArchiveMove {
                            old_title: parsed_data.old_title.clone(),
                            new_title: parsed_data.title.clone(),
                        })
                        .await
                        .unwrap();
                } else {
                    queue
                        .push(Message::Archive {
                            url: url.into(),
                            title: parsed_data.title.clone(),
                        })
                        .await
                        .unwrap();
                }
            }
        }
        match write(&wiki_location, &parsed_data).await {
            Ok(()) => {
                queue
                    .push(Message::Patch { patch: parsed_data })
                    .await
                    .unwrap();
                Ok(redir_uri.parse::<Uri>().unwrap())
            }
            Err(e) => {
                eprintln!("{}", e);
                Ok(Uri::from_static("/error"))
            }
        }
    }

    pub async fn append(
        form_body: HashMap<String, String>,
        wiki_location: String,
        queue: QueueHandle,
    ) -> Result<Uri, std::io::Error> {
        let parsed_data = form_body.get("body").unwrap();
        match create_journal_entry(&wiki_location, parsed_data.to_string()).await {
            Ok(patch) => {
                queue.push(Message::Patch { patch }).await.unwrap();
                Ok("/".parse::<Uri>().unwrap())
            }
            Err(e) => {
                eprintln!("{}", e);
                Ok(Uri::from_static("/error"))
            }
        }
    }

    pub async fn delete(queue: QueueHandle, form_body: HashMap<String, String>) -> Uri {
        let title = form_body.get("title").unwrap();
        queue
            .push(Message::Delete {
                title: title.into(),
            })
            .await
            .unwrap();

        purge_mru_cache(title).await;
        Uri::from_static("/")
    }
}

pub struct WikiPageRouter {
    pub parts: RefHubParts,
    pub wiki_location: Arc<String>,
}

impl WikiPageRouter {
    pub fn routes(&self) -> BoxedFilter<(impl Reply,)> {
        self.get_nested()
            .or(self.delete())
            .or(self.edit())
            .or(self.quick_add())
            .or(self.new_page())
            .or(self.backlink_index())
            .or(self.get())
            .boxed()
    }

    fn backlink_index(&self) -> BoxedFilter<(impl Reply,)> {
        let (links, _) = &self.parts;
        warp::get()
            .and(with_auth())
            .and(warp::path("links"))
            .and(with_links(links.to_owned()))
            .then(|links: GlobalBacklinks| async move {
                let response = Runner::render_backlink_index(links).await;
                warp::reply::html(response)
            })
            .boxed()
    }

    fn get(&self) -> BoxedFilter<(impl Reply,)> {
        let (links, _) = &self.parts;
        warp::get()
            .and(with_auth())
            .and(warp::path::param())
            .and(with_links(links.clone()))
            .and(with_location(self.wiki_location.clone()))
            .and(warp::query::<HashMap<String, String>>())
            .then(
                |path: String,
                 reflinks: GlobalBacklinks,
                 wiki_location: String,
                 query_params: HashMap<String, String>| async move {
                    let runner = Runner {};
                    let response = runner
                        .render_file(path, reflinks, wiki_location, query_params)
                        .await;
                    warp::reply::html(response)
                },
            )
            .boxed()
    }

    fn get_nested(&self) -> BoxedFilter<(impl Reply,)> {
        let (links, _) = &self.parts;
        warp::get()
            .and(with_auth())
            .and(warp::path!(String / String))
            .and(with_links(links.to_owned()))
            .and(with_location(self.wiki_location.clone()))
            .then(
                |main_path: String,
                 sub_path: String,
                 reflinks: GlobalBacklinks,
                 wiki_location: String| async move {
                    let response =
                        Runner::render_nested_file(main_path, sub_path, reflinks, wiki_location)
                            .await;
                    warp::reply::html(response)
                },
            )
            .boxed()
    }

    fn delete(&self) -> BoxedFilter<(impl Reply,)> {
        let (_, queue) = &self.parts;
        warp::post()
            .and(with_auth())
            .and(warp::path("delete"))
            .and(with_queue(queue.to_owned()))
            .and(warp::body::content_length_limit(MAX_BODY_SIZE))
            .and(warp::body::form())
            .then(
                |queue: QueueHandle, form_body: HashMap<String, String>| async {
                    let response = Runner::delete(queue, form_body).await;
                    warp::redirect(response)
                },
            )
            .boxed()
    }

    fn new_page(&self) -> BoxedFilter<(impl Reply,)> {
        warp::get()
            .and(with_auth())
            .and(
                warp::path("new")
                    .and(warp::query::<HashMap<String, String>>())
                    .then(|query_params: HashMap<String, String>| async {
                        let response = Runner::render_new(query_params).await;
                        warp::reply::html(response)
                    }),
            )
            .boxed()
    }

    fn edit(&self) -> BoxedFilter<(impl Reply,)> {
        let (_, queue) = &self.parts;
        warp::post()
            .and(with_auth())
            .and(
                warp::path("edit").and(
                    warp::body::content_length_limit(MAX_BODY_SIZE)
                        .and(warp::body::form())
                        .and(with_location(self.wiki_location.clone()))
                        .and(with_queue(queue.to_owned()))
                        .and(warp::query::<HashMap<String, String>>())
                        .then(
                            |form_body: HashMap<String, String>,
                             wiki_location: String,
                             queue: QueueHandle,
                             query_params: HashMap<String, String>| async {
                                let redir_url =
                                    Runner::edit(form_body, wiki_location, queue, query_params)
                                        .await
                                        .unwrap();
                                warp::redirect(redir_url)
                            },
                        ),
                ),
            )
            .boxed()
    }

    fn quick_add(&self) -> BoxedFilter<(impl Reply,)> {
        let (_, queue) = &self.parts;
        warp::post()
            .and(with_auth())
            .and(
                warp::path("quick-add").and(
                    warp::body::content_length_limit(MAX_BODY_SIZE)
                        .and(warp::body::form())
                        .and(with_location(self.wiki_location.clone()))
                        .and(with_queue(queue.to_owned()))
                        .then(
                            |form_body: HashMap<String, String>,
                             wiki_location: String,
                             queue: QueueHandle| async {
                                let response = Runner::append(form_body, wiki_location, queue)
                                    .await
                                    .unwrap();
                                warp::redirect(response)
                            },
                        ),
                ),
            )
            .boxed()
    }
}
