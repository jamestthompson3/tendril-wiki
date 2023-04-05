use std::collections::HashMap;
use std::fmt::Write as _;

use persistance::fs::{create_journal_entry, read, write, ReadPageError, WriteWikiError};
use render::{injected_html::InjectedHTML, new_page::NewPage, wiki_page::WikiPage, Render};
use urlencoding::decode;
use wikitext::{parsers::Note, GlobalBacklinks, PatchData};

use crate::{cache::purge_mru_cache, messages::Message, Queue, QueueHandle};

pub struct WikiRunner {}

impl WikiRunner {
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
        let templatted = note.to_template();
        let link_vals = reflinks.lock().await;
        let links = link_vals.get(&templatted.page.title);
        match note.header.get("content-type") {
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

    pub async fn edit(body: PatchData, queue: QueueHandle) -> Result<(), WriteWikiError> {
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
                queue.push(Message::Patch { patch: body }).await.unwrap();
                Ok(())
            }
            Err(e) => {
                eprintln!("{}", e);
                Err(e)
            }
        }
    }

    pub async fn append(body: PatchData, queue: QueueHandle) -> Result<(), WriteWikiError> {
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

    pub async fn delete(queue: QueueHandle, form_body: HashMap<String, String>) -> String {
        let title = form_body.get("title").unwrap();
        queue
            .push(Message::Delete {
                title: title.into(),
            })
            .await
            .unwrap();

        purge_mru_cache(title).await;
        String::from("/")
    }
}
