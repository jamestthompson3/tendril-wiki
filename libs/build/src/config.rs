use std::{
    env::var,
    fs,
    io::{stdin, stdout, Write},
    path::PathBuf,
};

use directories::ProjectDirs;
use rpassword::read_password_from_tty;
use serde_derive::{Deserialize, Serialize};
use tasks::parse_location;

pub type ConfigOptions = (PathBuf, PathBuf, bool, Option<String>, String, Option<String>);

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Sync {
    pub use_git: bool,
    pub sync_interval: u8,
    pub branch: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct General {
    pub wiki_location: String,
    pub port: u16,
    pub user: String,
    pub pass: String,
    pub version: String,
    pub media_location: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    pub general: General,
    pub sync: Sync,
}

pub fn get_config_location() -> (PathBuf, PathBuf) {
    let project_dir = ProjectDirs::from("", "", "tendril").unwrap();
    let config_dir = project_dir.config_dir();
    let mut config_path = PathBuf::from(config_dir);
    config_path.push("config.toml");
    (config_dir.to_owned(), config_path)
}

pub fn get_data_dir_location() -> PathBuf {
    let project_dir = ProjectDirs::from("", "", "tendril").unwrap();
    let data_dir = project_dir.data_dir();
    data_dir.to_owned()
}

fn prompt(f: Box<dyn Fn()>) -> String {
    let mut response = String::new();
    let stdin = stdin();
    f();
    stdout().flush().unwrap();
    stdin.read_line(&mut response).unwrap();
    response
}

pub fn gen_config_interactive() -> ConfigOptions {
    let username = prompt(Box::new(|| {
        print!("Enter username ({}):  ", get_user());
    }));
    let location = prompt(Box::new(|| {
        print!("Enter wiki location (~/wiki/) > ");
    }));
    let media_location = prompt(Box::new(|| {
        print!("Enter location for uploaded media (~/wiki_media/) > ");
    }));
    let should_sync = prompt(Box::new(|| {
        print!("Use git to sync wiki updates (y\\n)? ");
    }));
    let mut enable_sync = true;
    let mut git_branch: Option<String> = None;
    match should_sync.as_str().strip_suffix('\n').unwrap() {
        "y" | "t" | "true" | "yes" => {
            git_branch = Some(prompt(Box::new(|| {
                print!("Name of branch to sync to (main): ");
            })));
        }
        _ => enable_sync = false,
    }
    let use_password = prompt(Box::new(|| {
        print!("Use password to protect wiki (y\\n)? ");
    }));
    let mut password: Option<String> = None;
    match use_password.as_str().strip_suffix('\n').unwrap() {
        "true" | "y" | "t" | "yes" | "\n" => {
            password = Some(read_password_from_tty(Some("Password: ")).unwrap());
        }
        _ => {}
    }
    let parsed_location: PathBuf;
    if location == "\n" {
        parsed_location = parse_location("~/wiki/");
    } else {
        parsed_location = parse_location(location.strip_suffix('\n').unwrap_or(&location));
    }
    let parsed_media_location: PathBuf;
    if media_location == "\n" {
        parsed_media_location = parse_location("~/wiki_media/");
    } else {
        parsed_media_location =
            parse_location(media_location.strip_suffix('\n').unwrap_or(&location));
    }
    let mut branch: Option<String> = None;
    if let Some(git_branch) = git_branch {
    if git_branch == "\n" {
        branch = Some("main".into());
    } else {
        branch = Some(git_branch
            .strip_suffix('\n')
            .unwrap_or(&git_branch)
            .to_owned());
    }
    }
    let user: String;
    if username == "\n" {
        user = get_user();
    } else {
        user = username.strip_suffix('\n').unwrap_or(&username).to_owned();
    }
    (
        parsed_location,
        parsed_media_location,
        enable_sync,
        branch,
        user,
        password,
    )
}

pub fn read_config() -> Config {
    let (_, file) = get_config_location();
    let config: Config = toml::from_str(&fs::read_to_string(file).unwrap()).unwrap();
    config
}

fn get_user() -> String {
    match var("NAME") {
        Ok(user) => user,
        Err(_) => match var("USER") {
            Ok(user) => user,
            Err(_) => match var("USERNAME") {
                Ok(user) => user,
                Err(_) => String::from("user"),
            },
        },
    }
}
