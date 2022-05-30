use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
    time::Duration,
};

use build::{
    build_tags_and_links, delete_from_global_store, rename_in_global_store, update_global_store,
    update_mru_cache,
};
use futures::{stream, StreamExt};
use persistance::fs::{get_file_path, move_archive, path_to_data_structure, write, write_archive};
use regex::Regex;
use search_engine::{
    delete_archived_file, delete_entry_from_update, patch_search_from_archive,
    patch_search_from_update,
};
use tasks::{
    archive::{compress, extract},
    messages::{Message, PatchData},
    JobQueue, Queue,
};
use tokio::sync::Mutex;
use tokio::time::sleep;

const NUM_JOBS: u32 = 50;

lazy_static! {
    static ref TITLE_RGX: Regex = Regex::new(r"\?|\\|/|\||:|;|>|<|,|\.").unwrap();
}
pub async fn process_tasks(
    queue: Arc<JobQueue>,
    location: String,
    links: Arc<Mutex<BTreeMap<String, Vec<String>>>>,
) {
    loop {
        let jobs = match queue.pull(NUM_JOBS).await {
            Ok(jobs) => jobs,
            Err(err) => {
                eprintln!("{}", err);
                panic!("Failed to pull jobs");
            }
        };
        stream::iter(jobs)
            .for_each_concurrent(NUM_JOBS as usize, |job| async {
                match job.message {
                    Message::Rebuild => {
                        build_tags_and_links(&location, links.clone()).await;
                    }
                    Message::Patch { patch } => {
                        let note = patch.clone().into();

                        update_global_store(&patch.title, &note, links.clone()).await;
                        patch_search_from_update(&note).await;

                        if !patch.old_title.is_empty() && patch.old_title != patch.title {
                            rename_in_global_store(
                                &patch.title,
                                &patch.old_title,
                                &location,
                                links.clone(),
                            )
                            .await;
                        }
                        update_mru_cache(&patch.old_title, &patch.title).await;
                    }
                    Message::Delete { title } => {
                        let path = get_file_path(&location, &title).unwrap_or_else(|_| {
                            panic!("Failed to find file for deletion: {}", title)
                        });
                        let note = path_to_data_structure(&path).await.unwrap();
                        persistance::fs::delete(&location, &title).await.unwrap();
                        delete_from_global_store(&title, &note, links.clone()).await;
                        delete_entry_from_update(&title).await;
                        delete_archived_file(&title).await;
                    }
                    Message::Archive { url, title } => {
                        let product = tokio::task::spawn_blocking(|| extract(url)).await.unwrap();
                        let compressed = compress(&product.text);
                        write_archive(compressed, &title).await;
                        patch_search_from_archive((title, product.text)).await;
                    }
                    Message::ArchiveMove {
                        old_title,
                        new_title,
                    } => {
                        move_archive(old_title, new_title).await;
                    }
                    Message::NewFromUrl { url, tags } => {
                        let mut metadata = HashMap::new();
                        metadata.insert(String::from("url"), url.clone());
                        let product = tokio::task::spawn_blocking(move || extract(url))
                            .await
                            .unwrap();
                        let note_title = TITLE_RGX.replace(&product.title, "").to_string();
                        let compressed = compress(&product.text);
                        write_archive(compressed, &note_title).await;
                        patch_search_from_archive((note_title.clone(), product.text)).await;
                        let patch = PatchData {
                            body: String::with_capacity(0),
                            tags,
                            title: note_title.clone(),
                            old_title: String::with_capacity(0),
                            metadata,
                        };
                        write(&location, &patch).await.unwrap();
                        update_mru_cache(&patch.old_title, &patch.title).await;
                    }
                }
            })
            .await;
        sleep(Duration::from_millis(10)).await;
    }
}
