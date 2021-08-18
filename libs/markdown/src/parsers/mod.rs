pub mod html;
pub mod machine;
pub mod meta;
pub mod templates;

use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
    path::Path,
};

pub use self::html::*;
pub use self::meta::*;
pub use self::templates::*;

pub trait Reader {
    fn read(&self, location: &str) -> String;
}

// TODO: move somewhere else... Maybe another crate?
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

pub struct FileReader {}

impl Reader for FileReader {
    fn read(&self, location: &str) -> String {
        let mut file = File::open(location).unwrap();
        let mut markdown = String::new();
        file.read_to_string(&mut markdown).unwrap();
        markdown
    }
}
