use persistance::fs::{read_note_cache, write_note_cache};

pub async fn purge_mru_cache(title: &str) {
    let recent = read_note_cache().await;
    write_filtered_cache_file(filter_cache_file(&recent, title)).await;
}

pub async fn update_mru_cache(old_title: &str, current_title: &str) {
    let recent = read_note_cache().await;
    // Filter out the current title and the old title.
    // We don't need to separate based whether or not the not has been renamed since the
    // array is only ever 8 entries long, this will be fast.
    let filtered = filter_cache_file(&recent, current_title);
    let mut filtered = filter_cache_file(&filtered.join("\n"), old_title);
    if filtered.len() >= 8 {
        filtered.pop();
    }
    filtered.insert(0, current_title.into());
    write_filtered_cache_file(filtered).await;
}

async fn write_filtered_cache_file(filtered: Vec<String>) {
    let filtered = filtered.join("\n");
    write_note_cache(filtered).await;
}

fn filter_cache_file(recent: &str, title: &str) -> Vec<String> {
    recent
        .lines()
        .filter(|&line| line != title)
        .map(|l| l.to_owned())
        .collect::<Vec<String>>()
}
