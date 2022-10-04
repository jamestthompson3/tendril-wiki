use std::{collections::HashMap, sync::Arc};

use build::Titles;
use persistance::fs::utils::get_config_location;
use render::{
    all_pages::PageList, error_page::ErrorPage, file_upload_page::FileUploader,
    help_page::HelpPage, index_page::IndexPage, opensearch_page::OpenSearchPage,
    styles_page::StylesPage, uploaded_files_page::UploadedFilesPage, GlobalBacklinks, Render,
};
use tokio::fs::{self, read_dir};
use warp::{filters::BoxedFilter, Filter, Reply};

use crate::handlers::filters::with_location;

use super::{
    filters::{with_auth, with_host, with_links, with_user},
    with_titles,
};

struct Runner {}

impl Runner {
    pub async fn list_files(media_location: String) -> String {
        let mut entry_list = Vec::new();
        let mut entries = read_dir(media_location).await.unwrap();
        while let Ok(entry) = entries.next_entry().await {
            if entry.is_some() {
                let entry = entry.unwrap();
                entry_list.push(entry.file_name().into_string().unwrap());
            } else {
                break;
            }
        }
        let ctx = UploadedFilesPage {
            entries: entry_list,
        };
        ctx.render().await
    }
    pub async fn render_styles() -> String {
        let (path, _) = get_config_location();
        let style_location = path.join("userstyles.css");
        let body = fs::read_to_string(style_location).await.unwrap();
        let body = body.replace('\n', "\r\n");
        let ctx = StylesPage { body };
        ctx.render().await
    }
    pub async fn show_error(params: HashMap<String, String>) -> String {
        let msg = params
            .get("msg")
            .unwrap_or(&String::from("Error could not be determined."))
            .to_string();
        let ctx = ErrorPage { msg };
        ctx.render().await
    }
}

pub struct StaticPageRouter {
    user: Arc<String>,
    media_location: Arc<String>,
    host: Arc<String>,
    links: GlobalBacklinks,
    note_titles: Titles,
}

impl StaticPageRouter {
    pub fn new(
        user: Arc<String>,
        media_location: Arc<String>,
        host: Arc<String>,
        links: GlobalBacklinks,
        titles: Titles,
    ) -> Self {
        Self {
            user,
            media_location,
            host,
            links,
            note_titles: titles,
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
            .and(with_titles(self.note_titles.to_owned()))
            .then(|links: GlobalBacklinks, titles: Titles| async move {
                let links = links.lock().await;
                let titles = titles.lock().await;
                let mut name_and_count: Vec<(&String, usize)> = Vec::with_capacity(titles.len());
                for title in titles.iter() {
                    if let Some(link_list) = links.get(title) {
                        name_and_count.push((title, link_list.len()));
                    } else {
                        name_and_count.push((title, 0));
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
                let response = Runner::render_styles().await;
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
                let response = Runner::list_files(location).await;
                warp::reply::html(response)
            })
            .boxed()
    }
    fn error(&self) -> BoxedFilter<(impl Reply,)> {
        warp::get()
            .and(warp::path("error"))
            .and(warp::query::<HashMap<String, String>>())
            .then(move |params: HashMap<String, String>| async move {
                let response = Runner::show_error(params).await;
                warp::reply::html(response)
            })
            .boxed()
    }
}
