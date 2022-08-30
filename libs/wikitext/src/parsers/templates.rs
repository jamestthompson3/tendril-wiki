use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;

#[derive(Debug)]
pub struct TemplattedPage {
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
    pub raw_md: String,
    pub metadata: HashMap<String, String>,
}

pub struct ParsedTemplate {
    pub outlinks: Vec<String>,
    pub page: TemplattedPage,
}

pub type ParsedPages = Arc<Mutex<Vec<TemplattedPage>>>;
