use async_recursion::async_recursion;
use futures::{stream, StreamExt};

use render::static_site_page::StaticSitePage;
use wikitext::{
    parsers::{ParsedPages, TemplattedPage},
    GlobalBacklinks,
};

use persistance::fs::path_to_data_structure;
use persistance::fs::utils::get_config_location;
use render::Render;
use tokio::sync::Mutex;
use wikitext::processors::update_templatted_pages;

use std::{
    collections::{BTreeMap, HashMap},
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
        let pages = Arc::clone(&self.pages);
        write_entries(&pages, &self.backlinks).await;
        write_index_page(&pages).await;
        let mut config_dir = get_config_location().0;
        config_dir.push("userstyles.css");
        fs::create_dir("public/static").unwrap();
        fs::create_dir("public/config").unwrap();
        fs::copy("./static/style.css", "./public/static/style.css").unwrap();
        fs::copy("./static/mobile.css", "./public/static/mobile.css").unwrap();
        fs::copy(
            "./static/note-styles.css",
            "./public/static/note-styles.css",
        )
        .unwrap();
        if config_dir.exists() {
            fs::copy(config_dir, "./public/config/userstyles.css").unwrap();
        }
    }

    pub async fn sweep(&self, wiki_location: &str) {
        if !Path::new("./public").exists() {
            fs::create_dir_all("./public").unwrap();
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
    let templatted = note.to_template();
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
        let file_name = entry.file_name();
        let file_name = file_name.to_str().unwrap();
        if entry.file_type().unwrap().is_file() && file_name.ends_with(".txt") {
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

async fn write_index_page(pages: &ParsedPages) {
    let page_vals = pages.lock().await;
    let pages: String = page_vals
        .iter()
        .map(|page| format!(r#"<li><a href="{}">{}</a></li>"#, page.title, page.title))
        .collect();
    let body = format!(
        r#"<h1>Pages</h1><ul style="margin: 1rem 0rem;">{}</ul>"#,
        pages
    );
    let page = TemplattedPage {
        title: String::from("Notebook Index"),
        body,
        tags: Vec::with_capacity(0),
        desc: String::from("list of all pages"),
        metadata: HashMap::with_capacity(0),
    };
    let output = StaticSitePage::new(&page, None).render().await;
    // TODO: Figure out static site index
    tokio::fs::write("public/index.html", output).await.unwrap();
}

async fn write_entries(pages: &ParsedPages, backlinks: &GlobalBacklinks) {
    let page_vals = pages.lock().await;
    let link_vals = backlinks.lock().await;
    for page in page_vals.iter() {
        let links = link_vals.get(&page.title);
        let output = StaticSitePage::new(page, links).render().await;
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
