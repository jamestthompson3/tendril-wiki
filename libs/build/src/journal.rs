use std::fs;

use chrono::Local;
use persistance::fs::get_file_path;

pub fn create_journal_entry(mut location: String, entry: String) -> Result<(), std::io::Error> {
    let now = Local::now();
    let daily_file = now.format("%Y-%m-%d").to_string();
    if let Ok(exists) = get_file_path(&location, &daily_file) {
        let mut entry_file = fs::read_to_string(exists.clone()).unwrap();
        entry_file.push_str(&format!("\n\n[{}] {}", now.format("%H:%M"), entry));
        println!("\x1b[38;5;47mdaily journal updated\x1b[0m");
        fs::write(exists, entry_file)
    } else {
        let docstring = format!(
            r#"
---
title: {}
tags: [daily notes]
created: {}
---
[{}] {}
    "#,
            daily_file,
            now.to_string(),
            now.format("%H:%M").to_string(),
            entry
        );
        location.push_str(&daily_file);
        location.push_str(".md");
        println!("\x1b[38;5;47mdaily journal updated\x1b[0m");
        fs::write(&location, docstring)
    }
}
