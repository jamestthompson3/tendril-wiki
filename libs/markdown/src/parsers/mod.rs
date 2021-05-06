pub mod html;
pub mod meta;
pub mod templates;

use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

pub use self::html::*;
pub use self::meta::*;
pub use self::templates::*;

// TODO: move these things somewhere else... Maybe another crate?
pub fn path_to_reader(path: &PathBuf) -> Result<impl Iterator<Item = String>, std::io::Error> {
    match File::open(&path) {
        Ok(fd) => {
            let reader = BufReader::new(fd);
            Ok(reader.lines().map(|line| line.unwrap()))
        }
        Err(e) => Err(e),
    }
}

pub fn parse_wiki_entry(wiki_location: &str) -> PathBuf {
    let entrypoint: PathBuf;
    if wiki_location.contains('~') {
        entrypoint = PathBuf::from(wiki_location.replace('~', &std::env::var("HOME").unwrap()));
    } else {
        entrypoint = PathBuf::from(wiki_location);
    }
    entrypoint
}
