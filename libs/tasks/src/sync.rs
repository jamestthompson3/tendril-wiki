use std::{
    fs::File,
    process::{exit, Command, Output},
    time::Duration, sync::Arc,
};

use tokio::task::spawn;
use tokio::time::sleep;

use crate::{messages::Message, JobQueue, Queue};

struct Git {
    repo_location: String,
}

impl Git {
    fn new(repo_location: String) -> Self {
        let expected_git_dir = format!("{}.git", repo_location);
        if let Ok(file) = File::open(expected_git_dir) {
            if let Ok(metadata) = file.metadata() {
                if metadata.is_dir() {
                    Git { repo_location }
                } else {
                    eprintln!("Git directory not found! Please either initialize a git repository in the wiki location, or disable git syncing your config file");
                    exit(1);
                }
            } else {
                eprintln!("Could not read filesystem metadata of wiki's git repository");
                exit(1);
            }
        } else {
            eprintln!("Wiki location is not a git repository! Please either initialize a git repository in the wiki location, or disable git syncing your config file");
            exit(1);
        }
    }
    fn git_cmd(&self, args: &[&str]) -> Output {
        Command::new("git")
            .args(args)
            .current_dir(&self.repo_location)
            .output()
            .unwrap()
    }
    fn status(&self) -> usize {
        let output = self.git_cmd(&["status", "-s"]);
        if output.status.success() {
            let out = output.stdout;

            match String::from_utf8(out) {
                Ok(s) => s.lines().count(),
                Err(e) => {
                    panic!("{}", e);
                }
            }
        } else {
            panic!("could not get status of git repository");
        }
    }
    fn add(&self) {
        let output = self.git_cmd(&["add", "."]);
        if !output.status.success() {
            panic!("Could not add files to git!");
        }
    }
    fn commit(&self) {
        let output = self.git_cmd(&["commit", "-am", "[AutoSync] - tendrilwiki"]);
        if !output.status.success() {
            panic!("Could not add files to git!");
        }
    }
    fn push(&self, branch: &str) {
        let output = self.git_cmd(&["push", "-u", "origin", branch]);
        if !output.status.success() {
            panic!("Could not push to remote repository!");
        }
    }
    fn pull(&self, branch: &str) {
        let output = self.git_cmd(&["pull", "--rebase", "origin", branch]);
        if !output.status.success() {
            panic!("Could not pull remote changes!");
        }
    }
    // Note: this will fall apart if there are merge conflicts!
    async fn sync(&self, sync_interval: u8, branch: String, queue: Arc<JobQueue>) {
        loop {
            let changed_file_count = self.status();
            if changed_file_count > 0 {
                println!("│");
                println!("└─> Syncing");
                self.add();
                self.commit();
                self.pull(&branch);
                self.push(&branch);
            }
            queue.push(Message::Rebuild).await.unwrap();
            sleep(Duration::from_secs(sync_interval.into())).await
        }
    }
}

pub async fn sync(wiki_location: &str, sync_interval: u8, branch: String, queue: Arc<JobQueue>) {
    let git = Git::new(wiki_location.to_owned());
    spawn(async move { git.sync(sync_interval, branch, queue).await });
}

pub fn git_update(wiki_location: &str, branch: String) {
    let git = Git::new(wiki_location.to_owned());
    git.add();
    git.commit();
    git.pull(&branch);
    git.push(&branch);
    println!("\x1b[38;5;47mchanges synced\x1b[0m");
}
