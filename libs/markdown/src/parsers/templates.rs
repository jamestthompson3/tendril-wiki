use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, Mutex},
};

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

pub type TagMapping = Arc<Mutex<BTreeMap<String, Vec<String>>>>;
pub type GlobalBacklinks = Arc<Mutex<BTreeMap<String, Vec<String>>>>;
pub type ParsedPages = Arc<Mutex<Vec<TemplattedPage>>>;
pub type PageTitles = Arc<Mutex<Vec<String>>>;
