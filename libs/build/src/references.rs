use std::{collections::BTreeMap, fs::read_dir, io, path::PathBuf, sync::Arc};

use async_recursion::async_recursion;
use futures::{stream, StreamExt};
use persistance::fs::{path_to_data_structure, utils::get_file_path};
use tokio::{fs, sync::Mutex};
use wikitext::{parsers::Note, GlobalBacklinks};

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
            && entry.file_name().to_str().unwrap().ends_with(".txt")
        {
            tokio::spawn(process_file(entry.path(), links))
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
    let templatted = note.to_template();
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

pub async fn update_global_store(current_title: &str, note: &Note, links: GlobalBacklinks) {
    let mut links = links.lock().await;
    let templatted = note.to_template();
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

pub async fn delete_from_global_store(title: &str, note: &Note, links: GlobalBacklinks) {
    let mut links = links.lock().await;
    let templatted = note.to_template();
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

pub async fn rename_in_global_store(
    current_title: &str,
    old_title: &str,
    backlinks: GlobalBacklinks,
) {
    let mut backlinks = backlinks.lock().await;
    let linked_pages = backlinks.get(old_title);
    if let Some(linked_pages) = linked_pages {
        stream::iter(linked_pages)
            .for_each(|page| async {
                let location = get_file_path(page).unwrap();
                match fs::read_to_string(&location).await {
                    Ok(raw_page) => {
                        let relinked_page = raw_page.replace(old_title, current_title);
                        fs::write(location, relinked_page).await.unwrap();
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

#[cfg(test)]
mod tests {
    use std::{env, fs};

    use persistance::fs::utils::get_file_path;

    use super::*;

    const TEST_DIR: &str = "/tmp/tendril-test/references/";

    fn init_temp_wiki(namespace: &str) {
        env::set_var("TENDRIL_WIKI_DIR", TEST_DIR);
        fs::create_dir_all(format!("{}{}", TEST_DIR, namespace)).unwrap();
        for entry in fs::read_dir("../wikitext/fixtures").unwrap() {
            let mut dest = PathBuf::from(TEST_DIR);
            let entry = entry.unwrap();
            let path = entry.path();
            dest.push(entry.file_name());
            fs::copy(&path, dest).unwrap();
        }
    }
    fn cp_file(src: &str, dest: &str) {
        let mut src_path = PathBuf::from(TEST_DIR);
        src_path.push(src.to_owned() + ".txt");
        let mut dest_path = PathBuf::from(TEST_DIR);
        dest_path.push(dest.to_owned() + ".txt");
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
        let path = get_file_path(title).unwrap();
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
        rename_in_global_store(new_title, title, links.clone()).await;
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
        let path = get_file_path(title).unwrap();
        let note = path_to_data_structure(&path).await.unwrap();
        delete_from_global_store(title, &note, links.clone()).await;
        let updated_links = links.lock().await;
        let entry = updated_links.get(title);
        assert_eq!(entry, None);
        teardown_temp_wiki("delete");
    }
}
