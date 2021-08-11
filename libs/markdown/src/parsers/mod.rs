pub mod html;
pub mod meta;
pub mod templates;
pub mod machine;

use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

pub use self::html::*;
pub use self::meta::*;
pub use self::templates::*;

// TODO: move somewhere else... Maybe another crate?
pub fn path_to_reader<P: AsRef<Path> + ?Sized>(path: &P) -> Result<impl Iterator<Item = String>, std::io::Error> {
    match File::open(&path) {
        Ok(fd) => {
            let reader = BufReader::new(fd);
            Ok(reader.lines().map(|line| line.unwrap()))
        }
        Err(e) => Err(e),
    }
}
