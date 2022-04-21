pub mod password;
pub mod sync;
use directories::UserDirs;
use std::{
    path::{Path, PathBuf, MAIN_SEPARATOR},
    process::exit,
};
use tokio::fs::read_to_string;

pub use self::password::*;
pub use self::sync::*;

pub fn parse_location(location: &str) -> PathBuf {
    let mut loc: String;
    if location.contains('~') {
        if let Some(dirs) = UserDirs::new() {
            let home_dir: String = dirs.home_dir().to_string_lossy().into();
            loc = location.replace('~', &home_dir);
        } else {
            loc = location.replace('~', &std::env::var("HOME").unwrap());
        }
    } else {
        loc = location.to_owned();
    }
    if !loc.ends_with(MAIN_SEPARATOR) {
        loc.push(MAIN_SEPARATOR)
    }
    PathBuf::from(loc)
}
pub fn normalize_wiki_location(wiki_location: &str) -> String {
    let location = parse_location(wiki_location);
    // Stop the process if the wiki location doesn't exist
    if !PathBuf::from(&location).exists() {
        exit(1);
    }
    location.to_string_lossy().into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, path::PathBuf};

    #[test]
    fn formats_wiki_location() {
        assert_eq!(parse_location("./wiki"), PathBuf::from("./wiki/"));
        env::set_var("HOME", "test");
        assert_eq!(parse_location("~/wiki"), PathBuf::from("test/wiki/"));
        assert_eq!(
            parse_location("/user/~/wiki"),
            PathBuf::from("/user/test/wiki/")
        );
    }
}

// TODO: this is really dependent on file system ops, won't be good if we change the storage
// backend.
pub async fn path_to_string<P: AsRef<Path> + ?Sized>(path: &P) -> Result<String, std::io::Error> {
    read_to_string(&path).await
}
