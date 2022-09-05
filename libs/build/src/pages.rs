use async_recursion::async_recursion;
use futures::{stream, StreamExt};

use render::link_page::LinkPage;
use render::wiki_page::WikiPage;
use wikitext::parsers::ParsedPages;

use persistance::fs::config::read_config;
use persistance::fs::path_to_data_structure;
use persistance::fs::utils::get_config_location;
use render::{GlobalBacklinks, Render};
use tokio::sync::Mutex;
use wikitext::processors::{to_template, update_templatted_pages};

use std::env;
use std::{
    collections::BTreeMap,
    fs::{self, read_dir},
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::build_global_store;

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
        let config_general = read_config().general;
        write_index_page(config_general.user, config_general.host).await;
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

async fn write_index_page(user: String, host: String) {
    // let ctx = IndexPage { user, host };
    // TODO: Figure out static site index
    tokio::fs::write("public/index.html", format!("{}{}", user, host))
        .await
        .unwrap();
}

async fn write_entries(pages: &ParsedPages, backlinks: &GlobalBacklinks) {
    let page_vals = pages.lock().await;
    let link_vals = backlinks.lock().await;
    for page in page_vals.iter() {
        let links = link_vals.get(&page.title);
        let output = WikiPage::new(page, links).render().await;
        let formatted_title = page.title.replace('/', "-");
        let out_dir = format!("public/{}", formatted_title);
        // TODO use path here instead of title? Since `/` in title can cause issues in fs::write
        tokio::fs::create_dir(&out_dir)
            .await
            .unwrap_or_else(|e| eprintln!("{:?}\nCould not create dir: {}", e, out_dir));
        let out_file = format!("public/{}/index.html", formatted_title);
        tokio::fs::write(&out_file, output)
            .await
            .unwrap_or_else(|e| eprintln!("{:?}\nCould not write file: {}", e, out_file));
    }
}

pub async fn write_backlinks(map: GlobalBacklinks) {
    let link_map = map.lock().await;
    let ctx = LinkPage {
        links: link_map.clone(),
    };
    tokio::fs::write("public/links/index.html", ctx.render().await)
        .await
        .unwrap();
}
