use std::path::Path;

pub(crate) mod archive;
pub(crate) mod notebook;

pub(crate) trait Proccessor {
    fn load(&mut self, location: &Path);
}
