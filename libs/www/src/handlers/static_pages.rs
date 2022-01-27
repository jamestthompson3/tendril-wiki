use std::{fs, sync::Arc};

use build::get_config_location;
use render::{
    file_upload_page::FileUploader, help_page::HelpPage, index_page::IndexPage,
    styles_page::StylesPage, Render,
};
use tasks::CompileState;
use warp::{filters::BoxedFilter, Filter, Reply};

use crate::{controllers::list_files, handlers::filters::with_location};

use super::filters::{with_auth, with_user};

pub struct StaticPageRouter {
    pub user: Arc<String>,
    pub media_location: Arc<String>,
}

impl StaticPageRouter {
    pub fn routes(&self) -> BoxedFilter<(impl Reply,)> {
        self.file_list()
            .or(self.upload())
            .or(self.help())
            .or(self.styles())
            .boxed()
    }

    fn help(&self) -> BoxedFilter<(impl Reply,)> {
        warp::get()
            .and(with_auth())
            .and(warp::path("help"))
            .map(|| {
                let ctx = HelpPage {};
                warp::reply::html(ctx.render(&CompileState::Dynamic))
            })
            .boxed()
    }
    pub fn index(&self) -> BoxedFilter<(impl Reply,)> {
        let user = self.user.clone();
        warp::get()
            .and(with_auth())
            .and(with_user(user.to_string()))
            .map(|user: String| {
                let idx_ctx = IndexPage { user };
                warp::reply::html(idx_ctx.render(&CompileState::Dynamic))
            })
            .boxed()
    }
    fn upload(&self) -> BoxedFilter<(impl Reply,)> {
        warp::get()
            .and(with_auth())
            .and(warp::path("upload"))
            .map(|| {
                let ctx = FileUploader {};
                warp::reply::html(ctx.render(&CompileState::Dynamic))
            })
            .boxed()
    }

    fn styles(&self) -> BoxedFilter<(impl Reply,)> {
        warp::path("styles")
            .and(warp::get().and(with_auth()).map(|| {
                let (path, _) = get_config_location();
                let style_location = path.join("userstyles.css");
                let body = fs::read_to_string(style_location).unwrap();
                let ctx = StylesPage { body };
                warp::reply::html(ctx.render(&CompileState::Dynamic))
            }))
            .boxed()
    }
    fn file_list(&self) -> BoxedFilter<(impl Reply,)> {
        warp::get()
            .and(with_auth())
            .and(warp::path!("files" / "list"))
            .and(with_location(self.media_location.clone()))
            .and_then(list_files)
            .boxed()
    }
}
