use async_recursion::async_recursion;
use futures::{stream, StreamExt};
use markdown::parsers::{path_to_data_structure, ParsedPages};

use markdown::processors::{to_template, update_templatted_pages};
use persistance::fs::{write_entries, write_index_page};
use render::{write_backlinks, GlobalBacklinks};
use tokio::sync::Mutex;

use std::env;
use std::{
    collections::BTreeMap,
    fs::{self, read_dir},
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{build_global_store, get_config_location, read_config};

/// ## TODO:
/// figure out how to encapsulate parse_entries and process_file better
/// ## NOTE:
/// Some gotchas to think about -> We're essentially keeping the whole wiki text in memory
/// which means that for very large wikis it can be a memory hog.
/// For the current size I test with ( a little over 600 pages ), it currently consumes 12MB of memory.
/// Not a huge issue, since we don't keep this in memory for serving pages, but would be nice to
/// get this down.
pub struct Builder {
    pub backlinks: GlobalBacklinks,
    pub pages: ParsedPages,
}

impl Builder {
    pub fn new() -> Self {
        Builder {
            backlinks: Arc::new(Mutex::new(BTreeMap::new())),
            pages: Arc::new(Mutex::new(Vec::new())),
        }
    }
    pub async fn compile_all(&self) {
        env::set_var("TENDRIL_COMPILE_STATIC", "true");
        let links = Arc::clone(&self.backlinks);
        let pages = Arc::clone(&self.pages);
        write_entries(&pages, &self.backlinks).await;
        write_backlinks(links).await;
        write_index_page(read_config().general.user).await;
        let mut config_dir = get_config_location().0;
        config_dir.push("userstyles.css");
        fs::create_dir("public/static").unwrap();
        fs::create_dir("public/config").unwrap();
        fs::copy("./static/style.css", "./public/static/style.css").unwrap();
        fs::copy(config_dir, "./public/config/userstyles.css").unwrap();
    }

    pub async fn sweep(&self, wiki_location: &str) {
        if !Path::new("./public").exists() {
            fs::create_dir_all("./public/tags").unwrap();
            fs::create_dir_all("./public/links").unwrap();
        }
        let links = Arc::clone(&self.backlinks);
        let pages = Arc::clone(&self.pages);
        parse_entries(PathBuf::from(wiki_location), links, pages).await;
    }
}

impl Default for Builder {
    fn default() -> Self {
        Builder::new()
    }
}

async fn process_file(path: PathBuf, backlinks: GlobalBacklinks, pages: ParsedPages) {
    let note = path_to_data_structure(&path).await.unwrap();
    let templatted = to_template(&note);
    build_global_store(
        &templatted.page.title,
        &templatted.outlinks,
        backlinks,
        &templatted.page.tags,
    )
    .await;
    update_templatted_pages(templatted.page, pages).await;
}

#[async_recursion]
async fn parse_entries(
    entrypoint: PathBuf,
    backlinks: GlobalBacklinks,
    rendered_pages: ParsedPages,
) {
    let entries = read_dir(entrypoint).unwrap();
    let pipeline = stream::iter(entries).for_each(|entry| async {
        let links = Arc::clone(&backlinks);
        let pages = Arc::clone(&rendered_pages);
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file()
            && entry.file_name().to_str().unwrap().ends_with(".md")
        {
            tokio::spawn(async move {
                process_file(entry.path(), links, pages).await;
            })
            .await
            .unwrap();
        } else if entry.file_type().unwrap().is_dir()
            && !entry.path().to_str().unwrap().contains(".git")
        {
            parse_entries(entry.path(), links, pages).await;
        }
    });
    pipeline.await
}
