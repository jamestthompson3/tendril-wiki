use std::{
    env::var,
    fs,
    io::{stdin, stdout, Write},
    path::PathBuf,
};

use directories::ProjectDirs;
use rpassword::read_password_from_tty;
use serde_derive::{Deserialize, Serialize};
use tasks::hash_password;

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

pub fn write_config() {
    let (dir, file) = get_config_location();
    if !file.exists() {
        fs::create_dir_all(dir).unwrap();
        let mut default_conf: Config =
            toml::from_str(&fs::read_to_string("./config/config.toml").unwrap()).unwrap();
        default_conf.general.user = get_user();
        fs::write(&file, toml::to_string(&default_conf).unwrap()).unwrap();
    }
}

pub fn write_config_interactive() {
    let mut username = String::new();
    let stdin = stdin();
    print!("Enter username ({}):  ", get_user());
    stdout().flush().unwrap();
    stdin.read_line(&mut username).unwrap();
    let mut location = String::new();
    print!("Enter wiki location (~/wiki/) >");
    stdout().flush().unwrap();
    stdin.read_line(&mut location).unwrap();
    let mut should_sync = String::new();
    let mut enable_sync = true;
    print!("Use git to sync wiki updates (y\\n)? ");
    stdout().flush().unwrap();
    stdin.read_line(&mut should_sync).unwrap();
    let mut git_branch = String::new();
    match should_sync.as_str().strip_suffix('\n').unwrap() {
        "y" | "t" | "true" | "yes" => {
            print!("Name of branch to sync to (main): ");
            stdout().flush().unwrap();
            stdin.read_line(&mut git_branch).unwrap();
        }
        _ => enable_sync = false,
    }
    let mut use_password = String::new();
    print!("Use password to protect wiki (y\\n)? ");
    stdout().flush().unwrap();
    stdin.read_line(&mut use_password).unwrap();
    let mut password: Option<String> = None;
    match use_password.as_str().strip_suffix('\n').unwrap() {
        "true" | "y" | "t" | "yes" | "\n" => {
            password = Some(read_password_from_tty(Some("Password: ")).unwrap());
        }
        _ => {}
    }
    let parsed_location: String;
    if location == "\n" {
        parsed_location = "~/wiki/".into();
    } else {
        parsed_location = location.strip_suffix('\n').unwrap_or(&location).to_owned();
    }
    let branch: String;
    if git_branch == "\n" {
        branch = "main".into();
    } else {
        branch = git_branch
            .strip_suffix('\n')
            .unwrap_or(&git_branch)
            .to_owned();
    }
    let user: String;
    if username == "\n" {
        user = get_user();
    } else {
        user = username.strip_suffix('\n').unwrap_or(&username).to_owned();
    }

    let (dir, file) = get_config_location();
    if !file.exists() {
        fs::create_dir_all(dir).unwrap();
        let mut default_conf: Config =
            toml::from_str(&fs::read_to_string("./config/config.toml").unwrap()).unwrap();
        default_conf.general.user = user;
        default_conf.general.wiki_location = parsed_location;
        default_conf.sync.use_git = enable_sync;
        default_conf.sync.branch = branch;
        if let Some(password) = password {
            let pass = hash_password(&password.as_bytes());
            default_conf.general.pass = pass;
        }
        fs::write(&file, toml::to_string(&default_conf).unwrap()).unwrap();
    }
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
