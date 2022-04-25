use std::{collections::HashMap, path::Path};
use async_trait::async_trait;

use crate::Doc;

pub(crate) mod notebook;
pub(crate) mod archive;

#[async_trait]
pub(crate) trait Proccessor {
    async fn load(&mut self, location: &Path);
    fn index(&self) -> HashMap<String, Vec<String>>;
    /// Converts a set of doc data structures into a searchable index.
    fn docs_to_idx(&self) -> HashMap<String, &Doc>;
}
