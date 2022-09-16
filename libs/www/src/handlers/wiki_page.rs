use std::collections::HashMap;
use std::fmt::Write as _;

use build::purge_mru_cache;
use persistance::fs::{create_journal_entry, read, write, ReadPageError, WriteWikiError};
use render::{
    injected_html::InjectedHTML, link_page::LinkPage, new_page::NewPage, wiki_page::WikiPage,
    GlobalBacklinks, Render,
};
use tasks::{
    messages::{Message, PatchData},
    Queue,
};
use urlencoding::decode;
use warp::{filters::BoxedFilter, hyper::Uri, Filter, Reply};
use wikitext::{parsers::Note, processors::to_template};

use crate::RefHubParts;

use super::{
    filters::{reply_on_result, with_auth, with_links, with_queue},
    QueueHandle, MAX_BODY_SIZE,
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
        query_params: HashMap<String, String>,
    ) -> String {
        let path = decode(&path).unwrap();
        self.render_from_path(path.to_string(), reflinks, query_params)
            .await
            .unwrap()
    }

    async fn note_to_html(&self, note: Note, reflinks: GlobalBacklinks) -> String {
        let templatted = to_template(&note);
        let link_vals = reflinks.lock().await;
        let links = link_vals.get(&templatted.page.title);
        match note.header.get("type") {
            Some(content_type) => {
                if content_type == "html" {
                    return InjectedHTML::new(&templatted.page, links).render().await;
                }
                WikiPage::new(&templatted.page, links).render().await
            }
            None => WikiPage::new(&templatted.page, links).render().await,
        }
    }

    pub async fn render_nested_file(
        &self,
        mut main_path: String,
        sub_path: String,
        links: GlobalBacklinks,
    ) -> Result<String, ReadPageError> {
        // I don't know why warp doesn't decode the sub path here...
        let sub_path_decoded = decode(&sub_path).unwrap();
        write!(main_path, "/{}", sub_path_decoded).unwrap();
        match read(main_path.clone()).await {
            Ok(note) => Ok(self.note_to_html(note, links).await),
            Err(ReadPageError::PageNotFoundError) => {
                let ctx = NewPage {
                    title: Some(urlencoding::decode(&sub_path).unwrap().into_owned()),
                    linkto: None,
                    action_params: None,
                };
                Ok(ctx.render().await)
            }
            e => {
                eprint!("{:?}", e);
                Err(ReadPageError::Unknown)
            }
        }
    }

    pub async fn render_from_path(
        &self,
        path: String,
        links: GlobalBacklinks,
        query_params: HashMap<String, String>,
    ) -> Result<String, ReadPageError> {
        match read(path.clone()).await {
            Ok(note) => Ok(self.note_to_html(note, links).await),
            Err(ReadPageError::PageNotFoundError) => {
                let ctx = NewPage {
                    title: Some(urlencoding::decode(&path).unwrap().into_owned()),
                    linkto: query_params.get("linkto"),
                    action_params: None,
                };
                Ok(ctx.render().await)
            }
            e => {
                eprint!("{:?}", e);
                Err(ReadPageError::Unknown)
            }
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
        body: PatchData,
        queue: QueueHandle,
    ) -> Result<(), WriteWikiError> {
        if body
            .tags
            .iter()
            .map(|t| t.to_lowercase())
            .any(|t| t == "bookmark")
        {
            if let Some(url) = body.metadata.get("url") {
                if body.old_title != body.title && !body.old_title.is_empty() {
                    queue
                        .push(Message::ArchiveMove {
                            old_title: body.old_title.clone(),
                            new_title: body.title.clone(),
                        })
                        .await
                        .unwrap();
                } else {
                    queue
                        .push(Message::Archive {
                            url: url.into(),
                            title: body.title.clone(),
                        })
                        .await
                        .unwrap();
                }
            }
        }
        match write(&body).await {
            Ok(()) => {
                queue
                    .push(Message::Patch { patch: body })
                    .await
                    .unwrap();
                Ok(())
            }
            Err(e) => {
                eprintln!("{}", e);
                Err(e)
            }
        }
    }

    pub async fn append(
        body: PatchData,
        queue: QueueHandle,
    ) -> Result<(), WriteWikiError> {
        match create_journal_entry(body.body).await {
            Ok(patch) => {
                queue.push(Message::Patch { patch }).await.unwrap();
                Ok(())
            }
            Err(e) => {
                eprintln!("{}", e);
                Err(persistance::fs::WriteWikiError::WriteError(e))
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
    parts: RefHubParts,
}

impl WikiPageRouter {
    pub fn new(parts: RefHubParts) -> Self {
        Self { parts }
    }
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
            .and(warp::query::<HashMap<String, String>>())
            .then(
                |path: String,
                 reflinks: GlobalBacklinks,
                 query_params: HashMap<String, String>| async move {
                    let runner = Runner {};
                    let response = runner
                        .render_file(path, reflinks, query_params)
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
            .then(
                |main_path: String, sub_path: String, reflinks: GlobalBacklinks| async move {
                    let runner = Runner {};
                    let main_path = decode(&main_path).unwrap().to_string();
                    let sub_path = decode(&sub_path).unwrap().to_string();
                    let response = runner
                        .render_nested_file(main_path, sub_path, reflinks)
                        .await;
                    warp::reply::html(response.unwrap())
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
                        .and(warp::body::json())
                        .and(with_queue(queue.to_owned()))
                        .then(
                            |body: PatchData, queue: QueueHandle| async {
                                reply_on_result(Runner::edit(body, queue).await)
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
                        .and(warp::body::json())
                        .and(with_queue(queue.to_owned()))
                        .then(
                            |body: PatchData, queue: QueueHandle| async {
                                reply_on_result(Runner::append(body, queue).await)
                            },
                        ),
                ),
            )
            .boxed()
    }
}
