use std::path::Path;

use async_trait::async_trait;

pub(crate) mod archive;
pub(crate) mod notebook;

#[async_trait]
pub(crate) trait Proccessor {
    async fn load(&mut self, location: &Path);
}
