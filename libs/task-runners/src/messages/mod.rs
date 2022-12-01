use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use wikitext::PatchData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    Patch {
        patch: PatchData,
    },
    Rebuild,
    Delete {
        title: String,
    },
    Archive {
        url: String,
        title: String,
    },
    ArchiveMove {
        old_title: String,
        new_title: String,
    },
    NewFromUrl {
        url: String,
        tags: Vec<String>,
    },
    ArchiveBody {
        title: String,
        body: String,
    },
    VerifyDataInstallation {
        dataset: Vec<String>,
        install_location: PathBuf
    }
}
