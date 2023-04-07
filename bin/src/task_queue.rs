use std::{collections::HashMap, sync::Arc, time::Duration};

use build::{build_links, delete_from_global_store, rename_in_global_store, update_global_store};
use futures::{stream, StreamExt};
use persistance::fs::{
    move_archive, path_to_data_structure,
    utils::{archive_file_exists, get_file_path},
    write, write_archive,
};
use regex::Regex;
use search_engine::{
    delete_archived_file, delete_entry_from_update, patch_search_from_archive,
    patch_search_from_update,
};
use task_runners::{
    archive::{compress, extract},
    cache::update_mru_cache,
    messages::Message,
    verify::verify_data_installation,
    JobQueue, Queue,
};
use tokio::time::sleep;
use wikitext::{processors::sanitize_html, GlobalBacklinks, PatchData};

const NUM_JOBS: u32 = 50;

lazy_static! {
    static ref TITLE_RGX: Regex = Regex::new(r"\?|\\|/|\||:|;|>|<|,|\.|\n|\$|&").unwrap();
}

pub async fn process_tasks(queue: Arc<JobQueue>, location: String, links: GlobalBacklinks) {
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
                        let mut links = links.lock().await;
                        links.clear();
                        links.extend(build_links(&location).await);
                    }
                    Message::Patch { patch } => {
                        let note = patch.clone().into();

                        update_global_store(&patch.title, &note, links.clone()).await;
                        patch_search_from_update(&note).await;

                        if !patch.old_title.is_empty() && patch.old_title != patch.title {
                            rename_in_global_store(&patch.title, &patch.old_title, links.clone())
                                .await;
                        }
                        update_mru_cache(&patch.old_title, &patch.title).await;
                    }
                    Message::Delete { title } => {
                        let path = get_file_path(&title).unwrap_or_else(|_| {
                            panic!("Failed to find file for deletion: {}", title)
                        });
                        let note = path_to_data_structure(&path).await.unwrap();
                        delete_from_global_store(&title, &note, links.clone()).await;
                        delete_entry_from_update(&title).await;
                        delete_archived_file(&title).await;
                        persistance::fs::delete(&title).await.unwrap();
                    }
                    Message::Archive { url, title } => {
                        let product = tokio::task::spawn_blocking(|| extract(url)).await.unwrap();
                        let compressed = compress(&product.text);
                        if !archive_file_exists(&title) {
                            write_archive(compressed, &title).await;
                            patch_search_from_archive((title, product.text)).await;
                        }
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
                        let note_title = TITLE_RGX.replace_all(&product.title, "").to_string();
                        let sanitized_content = sanitize_html(&product.content);
                        let compressed = compress(&product.text);
                        write_archive(compressed, &note_title).await;
                        patch_search_from_archive((note_title.clone(), product.text)).await;
                        metadata.insert("content-type".into(), "html".into());
                        let patch = PatchData {
                            body: sanitized_content,
                            tags,
                            title: note_title.clone(),
                            old_title: String::with_capacity(0),
                            metadata,
                        };
                        write(&patch).await.unwrap();
                        let note = patch.clone().into();
                        update_global_store(&patch.title, &note, links.clone()).await;
                        patch_search_from_update(&note).await;
                        update_mru_cache(&patch.old_title, &patch.title).await;
                    }
                    Message::ArchiveBody { title, body } => {
                        let compressed = compress(&body);
                        write_archive(compressed, &title).await;
                        patch_search_from_archive((title.clone(), body)).await;
                    }
                    Message::VerifyDataInstallation {
                        dataset,
                        install_location,
                    } => {
                        verify_data_installation(dataset, install_location).await;
                    }
                }
            })
            .await;
        sleep(Duration::from_millis(10)).await;
    }
}
