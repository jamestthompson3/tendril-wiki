use std::{collections::HashMap, io, time::Instant};

use bytes::Bytes;
use persistance::fs::{read, utils::get_config_location, write_media};
use render::{search_results_page::SearchResultsPage, Render};
use search_engine::{semantic_search, Tokens};
use thiserror::Error;
use urlencoding::decode;
use wikitext::parsers::Note;

pub struct APIRunner {}

#[derive(Error, Debug)]
pub enum FileError {
    #[error("Could not parse form body")]
    FormBodyRead,
    #[error("Could not write media")]
    FileWrite,
}

impl APIRunner {
    pub async fn file(filename: String, data: Vec<u8>) -> Result<(), FileError> {
        match write_media(&filename, &data).await {
            Ok(()) => Ok(()),
            Err(e) => {
                eprintln!("Could not write media: {}", e);
                Err(FileError::FileWrite)
            }
        }
    }

    pub async fn get_note(filename: String) -> Note {
        let path = decode(&filename).unwrap();
        match read(path.into()).await {
            Ok(note) => note,
            _ => panic!("Failed to read note {}", filename),
        }
    }

    pub async fn process_image(filename: String, bytes: Bytes) -> Result<(), io::Error> {
        write_media(&filename, bytes.as_ref()).await
    }

    pub async fn note_search(term: String) -> String {
        let now = Instant::now();
        let found_pages = semantic_search(&term).await;
        let num_results = found_pages.len();
        let ctx = SearchResultsPage {
            pages: found_pages,
            num_results,
            time: now.elapsed(),
        };
        ctx.render().await
    }

    // TODO: Better error handling
    pub async fn dump_search_index() -> Tokens {
        search_engine::dump_search_index().await.unwrap()
    }

    pub async fn update_styles(form_body: HashMap<String, String>) -> Result<(), io::Error> {
        let (path, _) = get_config_location();
        let style_location = path.join("userstyles.css");
        let body = form_body.get("body").unwrap();
        tokio::fs::write(style_location, body).await
    }
    pub fn get_version() -> String {
        env!("CARGO_PKG_VERSION").to_owned()
    }
}
