use persistance::fs::get_note_titles;
use render::{
    all_pages::PageList, file_upload_page::FileUploader, help_page::HelpPage,
    index_page::IndexPage, opensearch_page::OpenSearchPage, Render,
};
use std::{collections::HashMap, sync::Arc};
use task_runners::runners::static_page_runner::StaticPageRunner;
use warp::{filters::BoxedFilter, Filter, Reply};
use wikitext::GlobalBacklinks;

use crate::handlers::filters::with_location;

use super::filters::{with_auth, with_host, with_links, with_user};

pub struct StaticPageRouter {
    user: Arc<String>,
    media_location: Arc<String>,
    host: Arc<String>,
    links: GlobalBacklinks,
}

impl StaticPageRouter {
    pub fn new(
        user: Arc<String>,
        media_location: Arc<String>,
        host: Arc<String>,
        links: GlobalBacklinks,
    ) -> Self {
        Self {
            user,
            media_location,
            host,
            links,
        }
    }
    pub fn routes(&self) -> BoxedFilter<(impl Reply,)> {
        self.file_list()
            .or(self.upload())
            .or(self.all_pages())
            .or(self.help())
            .or(self.open_search())
            .or(self.styles())
            .or(self.error())
            .boxed()
    }

    fn help(&self) -> BoxedFilter<(impl Reply,)> {
        warp::get()
            .and(with_auth())
            .and(warp::path("help"))
            .then(|| async {
                let ctx = HelpPage {};
                warp::reply::html(ctx.render().await)
            })
            .boxed()
    }
    pub fn index(&self) -> BoxedFilter<(impl Reply,)> {
        let user = self.user.clone();
        let host = self.host.clone();
        warp::get()
            .and(with_auth())
            .and(with_user(user.to_string()))
            .and(with_host(host.to_string()))
            .and(with_links(self.links.to_owned()))
            .then(|user: String, host: String, links: GlobalBacklinks| async {
                let idx_ctx = IndexPage::new(user, host, links);
                warp::reply::html(idx_ctx.render().await)
            })
            .boxed()
    }
    fn all_pages(&self) -> BoxedFilter<(impl Reply,)> {
        warp::get()
            .and(with_auth())
            .and(warp::path("all_pages"))
            .and(with_links(self.links.to_owned()))
            .then(|links: GlobalBacklinks| async move {
                let links = links.lock().await;
                let titles = get_note_titles().unwrap();
                let mut name_and_count: Vec<(&String, usize)> = Vec::with_capacity(titles.len());
                for title in titles.iter() {
                    match links.get::<str>(title.as_str()) {
                        Some(link_list) => {
                            name_and_count.push((title, link_list.len()));
                        }
                        None => {
                            name_and_count.push((title, 0));
                        }
                    }
                }
                let idx_ctx = PageList::new(name_and_count);
                warp::reply::html(idx_ctx.render().await)
            })
            .boxed()
    }
    fn open_search(&self) -> BoxedFilter<(impl Reply,)> {
        let user = self.user.clone();
        let host = self.host.clone();
        warp::get()
            .and(warp::path("opensearchdescription.xml"))
            .and(with_user(user.to_string()))
            .and(with_host(host.to_string()))
            .then(|user: String, host: String| async {
                let idx_ctx = OpenSearchPage { user, host };
                warp::reply::with_header(
                    idx_ctx.render().await,
                    "Content-Type",
                    "application/opensearchdescription+xml",
                )
                .into_response()
            })
            .boxed()
    }
    fn upload(&self) -> BoxedFilter<(impl Reply,)> {
        warp::get()
            .and(with_auth())
            .and(warp::path("upload"))
            .then(|| async {
                let ctx = FileUploader {};
                warp::reply::html(ctx.render().await)
            })
            .boxed()
    }

    fn styles(&self) -> BoxedFilter<(impl Reply,)> {
        warp::path("styles")
            .and(warp::get().and(with_auth()).then(|| async {
                let response = StaticPageRunner::render_styles().await;
                warp::reply::html(response)
            }))
            .boxed()
    }
    fn file_list(&self) -> BoxedFilter<(impl Reply,)> {
        warp::get()
            .and(with_auth())
            .and(warp::path!("files" / "list"))
            .and(with_location(self.media_location.clone()))
            .then(move |location: String| async move {
                let response = StaticPageRunner::list_files(location).await;
                warp::reply::html(response)
            })
            .boxed()
    }
    fn error(&self) -> BoxedFilter<(impl Reply,)> {
        warp::get()
            .and(warp::path("error"))
            .and(warp::query::<HashMap<String, String>>())
            .then(move |params: HashMap<String, String>| async move {
                let response = StaticPageRunner::show_error(params).await;
                warp::reply::html(response)
            })
            .boxed()
    }
}
