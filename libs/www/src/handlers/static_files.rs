use std::{path::PathBuf, sync::Arc};

use persistance::fs::utils::get_config_location;
use warp::{filters::BoxedFilter, Filter, Reply};

use crate::get_static_dir;

pub struct StaticFileRouter {
    pub media_location: Arc<String>,
}

impl StaticFileRouter {
    pub fn routes(&self) -> BoxedFilter<(impl Reply,)> {
        self.files().or(self.styles()).boxed()
    }

    fn styles(&self) -> BoxedFilter<(impl Reply,)> {
        let (config_dir, _) = get_config_location();
        let user_stylesheet = config_dir.join("userstyles.css");
        warp::path("static")
            .and(warp::fs::dir(get_static_dir()).map(|res: warp::fs::File| {
                warp::reply::with_header(res, "service-worker-allowed", "/")
            }))
            .or(warp::path("config").and(warp::fs::file(user_stylesheet)))
            .boxed()
    }
    fn files(&self) -> BoxedFilter<(impl Reply,)> {
        let media_location = self.media_location.clone();
        warp::path("files")
            .and(warp::fs::dir(PathBuf::from(media_location.as_str())))
            .boxed()
    }
}
