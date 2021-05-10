pub mod sync;
pub mod search;
pub mod password;
use std::{path::{PathBuf, MAIN_SEPARATOR}, process::exit};

pub use self::sync::*;
pub use self::search::*;
pub use self::password::*;

#[inline]
fn parse_location(location: &str) -> String {
    let mut loc: String;
    if location.contains('~') {
        loc = location.replace('~', &std::env::var("HOME").unwrap());
    } else {
        loc = location.to_owned();
    }
    if !loc.ends_with(MAIN_SEPARATOR) {
        loc.push(MAIN_SEPARATOR)
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
    use std::env;

    #[test]
    fn formats_wiki_location() {
        assert_eq!(parse_location("./wiki"), String::from("./wiki/"));
        env::set_var("HOME", "test");
        assert_eq!(parse_location("~/wiki"), String::from("test/wiki/"));
        assert_eq!(parse_location("/user/~/wiki"), String::from("/user/test/wiki/"));

    }

}
