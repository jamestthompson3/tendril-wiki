use std::{fs, sync::Arc};

use build::get_config_location;
use markdown::parsers::{FileUploader, HelpPage, IndexPage, SearchPage, StylesPage};
use sailfish::TemplateOnce;
use warp::{Filter, Rejection, Reply};

use crate::{controllers::list_files, handlers::filters::with_location};

use super::filters::{with_auth, with_user};

pub struct StaticPageRouter {
    pub user: Arc<String>,
    pub media_location: Arc<String>,
}

impl StaticPageRouter {
    pub fn routes(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        self.file_list()
            .or(self.search())
            .or(self.upload())
            .or(self.help())
            .or(self.styles())
    }

    fn search(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::get()
            .and(with_auth())
            .and(warp::path("search"))
            .map(|| {
                let ctx = SearchPage {};
                warp::reply::html(ctx.render_once().unwrap())
            })
    }

    fn help(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::get()
            .and(with_auth())
            .and(warp::path("help"))
            .map(|| {
                let ctx = HelpPage {};
                warp::reply::html(ctx.render_once().unwrap())
            })
    }
    pub fn index(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let user = self.user.clone();
        warp::get()
            .and(with_auth())
            .and(with_user(user.to_string()))
            .map(|user: String| {
                let idx_ctx = IndexPage { user };
                warp::reply::html(idx_ctx.render_once().unwrap())
            })
    }
    fn upload(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::get()
            .and(with_auth())
            .and(warp::path("upload"))
            .map(|| {
                let ctx = FileUploader {};
                warp::reply::html(ctx.render_once().unwrap())
            })
    }

    fn styles(&self) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("styles").and(warp::get().and(with_auth()).map(|| {
            let (path, _) = get_config_location();
            let style_location = path.join("userstyles.css");
            let body = fs::read_to_string(style_location).unwrap();
            let ctx = StylesPage { body };
            warp::reply::html(ctx.render_once().unwrap())
        }))
    }
    fn file_list(
        &self,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::get()
            .and(with_auth())
            .and(warp::path!("files" / "list"))
            .and(with_location(self.media_location.clone()))
            .and_then(list_files)
    }
}
