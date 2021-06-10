use std::{path::PathBuf, sync::Arc};

use build::get_config_location;
use warp::{Filter, Rejection, Reply};

use crate::get_static_dir;

pub struct StaticFileRouter {
    pub media_location: Arc<String>,
}

impl StaticFileRouter {
    pub fn routes(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        self.files().or(self.styles())
    }

    fn styles(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let (config_dir, _) = get_config_location();
        let user_stylesheet = config_dir.join("userstyles.css");
        warp::path("static")
            .and(warp::fs::dir(get_static_dir()))
            .or(warp::path("config").and(warp::fs::file(user_stylesheet)))
    }
    fn files(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let media_location = self.media_location.clone();
        warp::path("files").and(warp::fs::dir(PathBuf::from(media_location.as_str())))
    }
}
