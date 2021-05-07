use std::process::Command;

pub fn check_sync(wiki_location: &str) {
    let location: String;
    if wiki_location.contains('~') {
        location = String::from(wiki_location).replace('~', &std::env::var("HOME").unwrap());
    } else {
        location = String::from(wiki_location);
    }
    let output = Command::new("git")
        .args(&["status", "-s"])
        .current_dir(location)
        .output()
        .unwrap();
    let out = if output.status.success() {
        output.stdout
    } else {
        output.stderr
    };

    match String::from_utf8(out) {
        Ok(s) => {
            let num_lines: Vec<&str> = s.lines().collect();
            if num_lines.len() > 0 {
                println!("│");
                println!("└─> Syncing");
            }
        }
        Err(e) => {
            eprint!("Could not parse output: {}", e);
        }
    };
}
