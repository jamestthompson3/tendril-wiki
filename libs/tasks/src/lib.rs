pub mod sync;
use std::{path::PathBuf, process::exit};

pub use self::sync::*;
pub mod search;
pub use self::search::*;

#[inline]
fn parse_location(location: &str) -> String {
    let mut loc: String;
    if location.contains('~') {
        loc = location.replace('~', &std::env::var("HOME").unwrap());
    } else {
        loc = location.to_owned();
    }
    // Backlog: Deal with Windows later
    if !loc.ends_with('/') {
        loc.push('/')
    }
    loc
}
pub fn normalize_wiki_location(wiki_location: &str) -> String {
    let location = parse_location(wiki_location);
    // Stop the process if the wiki location doesn't exist
    if !PathBuf::from(&location).exists() {
        exit(1);
    }
    location
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_wiki_location() {
        let loc = "./wiki";
        assert_eq!(parse_location(loc), String::from("./wiki/"))
    }
}
