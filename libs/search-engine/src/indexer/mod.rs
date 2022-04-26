use std::{collections::HashMap, path::Path};
use async_trait::async_trait;

use crate::Doc;

pub(crate) mod notebook;
pub(crate) mod archive;

#[async_trait]
pub(crate) trait Proccessor {
    async fn load(&mut self, location: &Path);
}
