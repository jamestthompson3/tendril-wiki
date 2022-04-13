use std::{collections::BTreeMap, fs::read_dir, io, path::PathBuf, sync::Arc};

use async_recursion::async_recursion;
use futures::{stream, StreamExt};
use markdown::{
    parsers::{path_to_data_structure, NoteMeta},
    processors::to_template,
};
use persistance::fs::{get_file_path, read_note_cache, write_note_cache};
use render::GlobalBacklinks;
use tokio::{
    fs,
    sync::{
        mpsc::{Receiver, Sender},
        Mutex,
    },
};

pub type RefHubTx = Sender<(String, String)>;
pub type RefHubRx = Receiver<(String, String)>;

#[derive(Debug)]
pub struct RefHub {
    pub backlinks: GlobalBacklinks,
}

impl RefHub {
    pub fn new() -> Self {
        RefHub {
            backlinks: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }
    pub fn links(&self) -> GlobalBacklinks {
        Arc::clone(&self.backlinks)
    }
}

impl Default for RefHub {
    fn default() -> Self {
        RefHub::new()
    }
}

// TODO: Reduce these duplicated functions, think of a better abstraction
#[async_recursion]
pub async fn parse_entries(entrypoint: PathBuf, backlinks: GlobalBacklinks) {
    let entries = read_dir(entrypoint).unwrap();
    let pipeline = stream::iter(entries).for_each(|entry| async {
        let links = Arc::clone(&backlinks);
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file()
            && entry.file_name().to_str().unwrap().ends_with(".md")
        {
            tokio::spawn(async move { process_file(entry.path(), links).await })
                .await
                .unwrap();
        } else if entry.file_type().unwrap().is_dir()
            && !entry.path().to_str().unwrap().contains(".git")
        {
            parse_entries(entry.path(), links).await;
        }
    });
    pipeline.await;
}

async fn process_file(path: PathBuf, backlinks: GlobalBacklinks) {
    let note = path_to_data_structure(&path).await.unwrap();
    let templatted = to_template(&note);
    build_global_store(
        &templatted.page.title,
        &templatted.outlinks,
        backlinks,
        &templatted.page.tags,
    )
    .await;
}

/// Updates a BTreeMap for tags and backlinks with each tag or link to a given note title acting as a key.
/// If we have a note titled `my_note` with the following body:
///
/// One great thing about wikis is `[[backlinks]]` that `[[connect|networked thought]]` your ideas!
///
/// We would take the vector consisting of the strings, `backlinks` and `networked though`, and
/// iterate through each entry, placing it as a key in the BTreeMap. This makes it easy to query
/// the map when we render a specific page since each value for that key will be the title of a
/// page that has a link to the currently viewed entry.
///
pub async fn build_global_store(
    title: &str,
    outlinks: &[String],
    backlinks: GlobalBacklinks,
    tags: &[String],
) {
    let mut global_backlinks = backlinks.lock().await;
    for link in outlinks.iter() {
        match global_backlinks.get_mut(link) {
            Some(links) => {
                links.push(title.to_owned());
            }
            None => {
                global_backlinks.insert(link.to_string(), vec![title.to_owned()]);
            }
        }
    }

    for tag in tags.iter() {
        match global_backlinks.get_mut(tag) {
            Some(tags) => {
                tags.push(title.to_owned());
            }
            None => {
                global_backlinks.insert(tag.to_string(), vec![title.to_owned()]);
            }
        }
    }
}

pub async fn build_tags_and_links(wiki_location: &str, links: GlobalBacklinks) {
    links.lock().await.clear();
    parse_entries(PathBuf::from(wiki_location), links.clone()).await;
}

pub async fn update_global_store(current_title: &str, note: &NoteMeta, links: GlobalBacklinks) {
    let mut links = links.lock().await;
    let templatted = to_template(note);
    for link in templatted.outlinks {
        match links.get_mut(&link) {
            Some(exists) => {
                if exists.contains(&String::from(current_title)) {
                    continue;
                } else {
                    exists.push(current_title.into())
                }
            }
            None => {
                links.insert(link, vec![current_title.into()]);
            }
        }
    }
    for tag in templatted.page.tags {
        match links.get_mut(&tag) {
            Some(exists) => {
                if exists.contains(&String::from(current_title)) {
                    continue;
                } else {
                    exists.push(current_title.into())
                }
            }
            None => {
                links.insert(tag, vec![current_title.into()]);
            }
        }
    }
}

pub async fn delete_from_global_store(title: &str, note: &NoteMeta, links: GlobalBacklinks) {
    let mut links = links.lock().await;
    let templatted = to_template(note);
    for link in templatted.outlinks {
        if let Some(exists) = links.get(&link) {
            if exists.contains(&String::from(title)) {
                let filtered = exists
                    .iter()
                    .filter(|&note| note != title)
                    .map(|n| n.to_owned())
                    .collect::<Vec<String>>();
                links.insert(link, filtered);
            }
        }
    }
    for tag in templatted.page.tags {
        if let Some(exists) = links.get(&tag) {
            if exists.contains(&String::from(title)) {
                let filtered = exists
                    .iter()
                    .filter(|&note| note != title)
                    .map(|n| n.to_owned())
                    .collect::<Vec<String>>();
                links.insert(tag, filtered);
            }
        }
    }
    links.remove(title);
}
pub async fn purge_file(location: &str, title: &str) {
    let recent = read_note_cache().await;
    write_filtered_cache_file(filter_cache_file(recent, title)).await;
    persistance::fs::delete(location, title).await.unwrap();
}

pub async fn update_mru_cache(old_title: &str, current_title: &str) {
    let recent = read_note_cache().await;
    // Filter out the current title and the old title.
    // We don't need to separate based whether or not the not has been renamed since the
    // array is only ever 8 entries long, this will be fast.
    let mut filtered = filter_cache_file(recent, current_title);
    filtered = filtered
        .iter()
        .filter(|&entry| entry != old_title)
        .map(|n| n.to_owned())
        .collect::<Vec<String>>();
    if filtered.len() >= 8 {
        filtered.pop();
    }
    filtered.insert(0, current_title.into());
    write_filtered_cache_file(filtered).await;
}

pub async fn rename_in_global_store(
    current_title: &str,
    old_title: &str,
    location: &str,
    backlinks: GlobalBacklinks,
) {
    let mut backlinks = backlinks.lock().await;
    let linked_pages = backlinks.get(old_title);
    if let Some(linked_pages) = linked_pages {
        stream::iter(linked_pages)
            .for_each(|page| async {
                let mut wiki_loc = PathBuf::from(location);
                let mut page = page.clone();
                page.push_str(".md");
                wiki_loc.push(&page);
                match fs::read_to_string(&wiki_loc).await {
                    Ok(raw_page) => {
                        let relinked_page = raw_page.replace(old_title, current_title);
                        fs::write(wiki_loc, relinked_page).await.unwrap();
                    }
                    Err(e) => match e.kind() {
                        io::ErrorKind::NotFound => {}
                        _ => std::panic::panic_any(e),
                    },
                }
            })
            .await;
        let pages = linked_pages.clone();
        backlinks.insert(current_title.into(), pages);
        backlinks.remove(old_title);
    }
}

fn filter_cache_file(recent: String, title: &str) -> Vec<String> {
    recent
        .lines()
        .filter(|&line| line != title)
        .map(|l| l.to_owned())
        .collect::<Vec<String>>()
}

async fn write_filtered_cache_file(filtered: Vec<String>) {
    let filtered = filtered.join("\n");
    write_note_cache(filtered).await;
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    const TEST_DIR: &str = "/tmp/tendril-test/references/";

    fn init_temp_wiki(namespace: &str) {
        fs::create_dir_all(format!("{}{}", TEST_DIR, namespace)).unwrap();
        for entry in fs::read_dir("../markdown/fixtures").unwrap() {
            let mut dest = PathBuf::from(TEST_DIR);
            let entry = entry.unwrap();
            let path = entry.path();
            dest.push(entry.file_name());
            fs::copy(&path, dest).unwrap();
        }
    }
    fn cp_file(src: &str, dest: &str) {
        let mut src_path = PathBuf::from(TEST_DIR);
        src_path.push(src.to_owned() + ".md");
        let mut dest_path = PathBuf::from(TEST_DIR);
        dest_path.push(dest.to_owned() + ".md");
        assert!(src_path.exists());
        fs::copy(src_path, dest_path).unwrap();
    }
    fn teardown_temp_wiki(namespace: &str) {
        for entry in fs::read_dir(format!("{}{}", TEST_DIR, namespace)).unwrap() {
            let entry = entry.unwrap();
            fs::remove_file(entry.path()).unwrap();
        }
    }

    #[tokio::test]
    // TODO: This is flaky
    #[ignore]
    async fn updates_note_succesfully() {
        init_temp_wiki("update");
        let title = "Logical reality";
        let mut link_tree = BTreeMap::new();
        link_tree.insert(title.into(), vec!["wiki page".into()]);
        let links: GlobalBacklinks = Arc::new(Mutex::new(link_tree));
        let path = get_file_path(&TEST_DIR, title).unwrap();
        let note = path_to_data_structure(&path).await.unwrap();
        update_global_store(title, &note, links.clone()).await;
        let updated_links = links.lock().await;
        let entry = updated_links.get(title).unwrap();
        assert_eq!(entry, &vec![String::from("wiki page")]);
        teardown_temp_wiki("update");
    }
    #[tokio::test]
    async fn renames_note_succesfully() {
        init_temp_wiki("rename");
        let title = "Logical reality";
        let new_title = "reality building";
        cp_file(title, new_title);
        let mut link_tree = BTreeMap::new();
        link_tree.insert(title.into(), vec!["wiki page".into()]);
        let links: GlobalBacklinks = Arc::new(Mutex::new(link_tree));
        rename_in_global_store(new_title, title, TEST_DIR, links.clone()).await;
        let updated_links = links.lock().await;
        let entry = updated_links.get(title);
        let renamed_entry = updated_links.get(new_title).unwrap();
        assert_eq!(entry, None);
        assert_eq!(renamed_entry, &vec![String::from("wiki page")]);
        teardown_temp_wiki("rename");
    }
    #[tokio::test]
    async fn deletes_from_global_store() {
        init_temp_wiki("delete");
        let title = "Logical reality";
        let mut link_tree = BTreeMap::new();
        link_tree.insert(title.into(), vec!["wiki page".into()]);
        let links: GlobalBacklinks = Arc::new(Mutex::new(link_tree));
        delete_from_global_store(title, TEST_DIR, links.clone()).await;
        let updated_links = links.lock().await;
        let entry = updated_links.get(title);
        assert_eq!(entry, None);
        teardown_temp_wiki("delete");
    }
}
