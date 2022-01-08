use std::{
    collections::BTreeMap,
    fs::read_dir,
    path::PathBuf,
    sync::{Arc, Mutex, MutexGuard},
};

use async_recursion::async_recursion;
use futures::{stream, StreamExt};
use markdown::{parsers::path_to_data_structure, processors::to_template};
use persistance::fs::{get_file_path, read_note_cache, write_note_cache};
use render::{GlobalBacklinks, TagMapping};
use tokio::sync::mpsc::{Receiver, Sender};

pub type RefHubTx = Sender<(String, String)>;
pub type RefHubRx = Receiver<(String, String)>;

#[derive(Debug)]
pub struct RefHub {
    pub tag_map: TagMapping,
    pub backlinks: GlobalBacklinks,
}

impl RefHub {
    pub fn new() -> Self {
        RefHub {
            tag_map: Arc::new(Mutex::new(BTreeMap::new())),
            backlinks: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }
    pub fn tags(&self) -> TagMapping {
        Arc::clone(&self.tag_map)
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
pub async fn parse_entries(entrypoint: PathBuf, tag_map: TagMapping, backlinks: GlobalBacklinks) {
    let entries = read_dir(entrypoint).unwrap();
    let pipeline = stream::iter(entries).for_each(|entry| async {
        let tags = Arc::clone(&tag_map);
        let links = Arc::clone(&backlinks);
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file()
            && entry.file_name().to_str().unwrap().ends_with(".md")
        {
            tokio::spawn(async move { process_file(entry.path(), tags, links) })
                .await
                .unwrap();
        } else if entry.file_type().unwrap().is_dir()
            && !entry.path().to_str().unwrap().contains(".git")
        {
            parse_entries(entry.path(), tags, links).await;
        }
    });
    pipeline.await;
}

fn process_file(path: PathBuf, tags: TagMapping, backlinks: GlobalBacklinks) {
    let note = path_to_data_structure(&path).unwrap();
    let templatted = to_template(&note);
    build_global_store(
        &templatted.page.title,
        &templatted.outlinks,
        backlinks,
        tags,
        &templatted.page.tags,
    );
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
pub fn build_global_store(
    title: &str,
    outlinks: &[String],
    backlinks: GlobalBacklinks,
    tag_map: TagMapping,
    tags: &[String],
) {
    let mut global_backlinks = backlinks.lock().unwrap();
    for link in outlinks.iter() {
        match global_backlinks.get_mut(&link.to_string()) {
            Some(links) => {
                links.push(title.to_owned());
            }
            None => {
                global_backlinks.insert(link.to_string(), vec![title.to_owned()]);
            }
        }
    }

    let mut tag_map = tag_map.lock().unwrap();
    for tag in tags.iter() {
        match tag_map.get_mut(&tag.to_string()) {
            Some(tags) => {
                tags.push(title.to_owned());
            }
            None => {
                tag_map.insert(tag.to_string(), vec![title.to_owned()]);
            }
        }
    }
}

pub async fn build_tags_and_links(wiki_location: &str, tags: TagMapping, links: GlobalBacklinks) {
    tags.lock().unwrap().clear();
    links.lock().unwrap().clear();
    parse_entries(PathBuf::from(wiki_location), tags.clone(), links.clone()).await;
}

pub fn update_global_store(title: &str, location: &str, links: GlobalBacklinks, tags: TagMapping) {
    if let [old_title, current_title] = title.split("~~").collect::<Vec<&str>>()[..] {
        let mut links = links.lock().unwrap();
        let mut tags = tags.lock().unwrap();
        // _should_ always be Ok(path)...
        if let Ok(path) = get_file_path(location, current_title) {
            let note = path_to_data_structure(&path).unwrap();
            let templatted = to_template(&note);
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
                match tags.get_mut(&tag) {
                    Some(exists) => {
                        if exists.contains(&String::from(current_title)) {
                            continue;
                        } else {
                            exists.push(current_title.into())
                        }
                    }
                    None => {
                        tags.insert(tag, vec![current_title.into()]);
                    }
                }
            }
        }
        if !old_title.is_empty() && old_title != current_title {
            rename_in_global_store(current_title, old_title, location, links);
        }
        let recent = read_note_cache();
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
        write_filtered_cache_file(filtered);
    }
}

pub fn delete_from_global_store(
    title: &str,
    location: &str,
    links: GlobalBacklinks,
    tags: TagMapping,
) {
    let mut links = links.lock().unwrap();
    let mut tags = tags.lock().unwrap();
    if let Ok(path) = get_file_path(location, title) {
        let note = path_to_data_structure(&path).unwrap();
        let templatted = to_template(&note);
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
            if let Some(exists) = tags.get(&tag) {
                if exists.contains(&String::from(title)) {
                    let filtered = exists
                        .iter()
                        .filter(|&note| note != title)
                        .map(|n| n.to_owned())
                        .collect::<Vec<String>>();
                    tags.insert(tag, filtered);
                }
            }
        }
    }

    let recent = read_note_cache();
    write_filtered_cache_file(filter_cache_file(recent, title));
    persistance::fs::delete(location, title).unwrap();
}

fn rename_in_global_store(
    current_title: &str,
    old_title: &str,
    location: &str,
    mut backlinks: MutexGuard<BTreeMap<String, Vec<String>>>,
) {
    let linked_pages = backlinks.get(old_title);
    if let Some(linked_pages) = linked_pages {
        IntoIterator::into_iter(linked_pages).for_each(|page| {
            let mut wiki_loc = String::from(location);
            let mut page = page.clone();
            page.push_str(".md");
            wiki_loc.push_str(&page);
            let raw_page = std::fs::read_to_string(&wiki_loc).unwrap();
            let relinked_page = raw_page.replace(old_title, current_title);
            std::fs::write(wiki_loc, relinked_page).unwrap();
        });
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

fn write_filtered_cache_file(filtered: Vec<String>) {
    let filtered = filtered.join("\n");
    write_note_cache(filtered);
}
