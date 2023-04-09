use std::{collections::BTreeMap, io, path::PathBuf, sync::Arc};

use async_recursion::async_recursion;
use futures::{stream, StreamExt};
use persistance::fs::{path_to_data_structure, utils::get_file_path};
use tokio::fs::{self, read_dir};
use wikitext::{parsers::Note, Backlinks, GlobalBacklinks};

// TODO: Reduce these duplicated functions, think of a better abstraction
#[async_recursion]
pub async fn parse_entries(entrypoint: PathBuf) -> Vec<(String, Vec<String>)> {
    let mut entries = read_dir(entrypoint).await.unwrap();
    let mut result = Vec::new();
    while let Some(entry) = entries.next_entry().await.unwrap() {
        if entry.file_type().await.unwrap().is_file()
            && entry.file_name().to_str().unwrap().ends_with(".txt")
        {
            let note = path_to_data_structure(&entry.path()).unwrap();
            let structured = note.to_structured();
            result.push(structured.as_owned());
        } else if entry.file_type().await.unwrap().is_dir()
            && !entry.path().to_str().unwrap().contains(".git")
        {
            let results = parse_entries(entry.path()).await;
            result.extend(results);
        }
    }
    result
}

async fn create_global_store(notes: Vec<(String, Vec<String>)>) -> Backlinks {
    let mut backlinks = BTreeMap::new();
    for note in notes {
        add_to_global_store(&note.0, &note.1, &mut backlinks).await;
    }
    backlinks
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
pub async fn add_to_global_store<'a>(
    title: &'a str,
    links_and_tags: &[String],
    backlinks: &mut Backlinks,
) {
    for link in links_and_tags.iter() {
        backlinks
            .entry(link.to_string())
            .or_default()
            .push(title.to_string());
    }
}

pub async fn build_links(wiki_location: Arc<String>) -> Backlinks {
    let entries = parse_entries(PathBuf::from(wiki_location.as_str())).await;
    create_global_store(entries).await
}

pub async fn update_global_store(current_title: &str, note: &Note, links: GlobalBacklinks) {
    let mut links = links.lock().await;
    let structured = note.to_structured();
    for link in structured.links_and_tags.iter() {
        links
            .entry(link.to_string())
            .or_default()
            .push(current_title.to_string());
    }
}

pub async fn delete_from_global_store(title: &str, note: &Note, links: GlobalBacklinks) {
    let mut links = links.lock().await;
    let templatted = note.to_template();
    for link in templatted.outlinks {
        let link = link.to_string();
        if let Some(exists) = links.get(&link) {
            if exists.contains(&title.to_string()) {
                let filtered = exists
                    .iter()
                    .filter(|&note| note != title)
                    .map(|n| n.into())
                    .collect();
                links.insert(link, filtered);
            }
        }
    }
    for tag in templatted.page.tags {
        let tag = tag.to_string();
        if let Some(exists) = links.get(&tag) {
            if exists.contains(&String::from(title)) {
                let filtered = exists
                    .iter()
                    .filter(|&note| note != title)
                    .map(|n| n.to_owned())
                    .collect();
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
    use std::sync::Arc;
    use std::{env, fs};
    use tokio::sync::Mutex;

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
        let note = path_to_data_structure(&path).unwrap();
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
        let note = path_to_data_structure(&path).unwrap();
        delete_from_global_store(title, &note, links.clone()).await;
        let updated_links = links.lock().await;
        let entry = updated_links.get(title);
        assert_eq!(entry, None);
        teardown_temp_wiki("delete");
    }
}
