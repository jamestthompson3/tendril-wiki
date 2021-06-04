use std::{collections::HashMap, sync::Arc};

use build::RefBuilder;
use warp::{Filter, Rejection, Reply};

use crate::handlers::filters::with_nested_file;

use super::filters::{with_auth, with_file, with_location, with_refs};

pub fn wiki(
    ref_builder: RefBuilder,
    location: Arc<String>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::get()
        .and(with_auth())
        .and(warp::path::param())
        .and(with_refs(ref_builder))
        .and(with_location(location))
        .and(warp::query::<HashMap<String, String>>())
        .and_then(with_file)
}

pub fn nested_file(
    ref_builder: RefBuilder,
    location: Arc<String>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::get()
        .and(with_auth())
        .and(warp::path!(String / String))
        .and(with_refs(ref_builder))
        .and(with_location(location))
        .and_then(with_nested_file)
}
