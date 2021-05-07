pub mod sync;
pub use self::sync::*;
pub mod search;
pub use self::search::*;

pub fn normalize_wiki_location(wiki_location: &str) -> String {
    let location: String;
    if wiki_location.contains('~') {
        location = wiki_location.replace('~', &std::env::var("HOME").unwrap());
    } else {
        location = wiki_location.to_owned();
    }
    location
}
