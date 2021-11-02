pub mod password;
pub mod search;
pub mod sync;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf, MAIN_SEPARATOR},
    process::exit,
};

pub use self::password::*;
pub use self::search::*;
pub use self::sync::*;

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
        assert_eq!(
            parse_location("/user/~/wiki"),
            String::from("/user/test/wiki/")
        );
    }
}

pub fn path_to_reader<P: AsRef<Path> + ?Sized>(
    path: &P,
) -> Result<impl Iterator<Item = String>, std::io::Error> {
    match File::open(&path) {
        Ok(fd) => {
            let reader = BufReader::new(fd);
            Ok(reader.lines().map(|line| line.unwrap()))
        }
        Err(e) => Err(e),
    }
}
