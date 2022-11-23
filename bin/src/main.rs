use build::{build_tags_and_links, install, migrate, pages::Builder, update, RefHub};
use persistance::fs::{
    config::read_config,
    create_journal_entry,
    utils::{get_config_location, get_data_dir_location, normalize_wiki_location},
};
use search_engine::build_search_index;
use std::{path::PathBuf, process::exit, sync::Arc, time::Instant};
use task_queue::process_tasks;
use task_runners::{git_update, sync, JobQueue};
use tokio::fs;
use www::server;

#[macro_use]
extern crate lazy_static;

mod task_queue;

#[tokio::main]
async fn main() {
    let args = std::env::args().skip(1).collect::<Vec<String>>();
    let mut build_all = false;
    if !args.is_empty() {
        let arg = args[0].as_str();
        match arg {
            "-v" | "--version" => return print_version(),
            "-h" | "--help" => return print_help(),
            "-b" | "--build" => build_all = true,
            "-i" | "--init" => return install(),
            "-u" | "--update" => return update(),
            "-m" | "--migrate" => return migrate(),
            _ => {
                if arg.starts_with('-') {
                    eprintln!("unknown option: {}", arg);
                    exit(1);
                }
                if !arg.is_empty() {
                    let config = read_config();
                    let location = normalize_wiki_location(&config.general.wiki_location);
                    create_journal_entry(args.join(" ")).await.unwrap();
                    if config.sync.use_git {
                        git_update(&location, config.sync.branch);
                    }
                    exit(0);
                }
            }
        }
    }
    let config = read_config();
    let location = normalize_wiki_location(&config.general.wiki_location);
    if build_all {
        let now = Instant::now();
        if PathBuf::from("./public").exists() {
            fs::remove_dir_all("./public").await.unwrap();
        }
        let builder = Builder::new();
        builder.sweep(&location).await;
        builder.compile_all().await;
        println!("Built static site in: {}ms", now.elapsed().as_millis());
    } else {
        let ref_hub = RefHub::new();
        let job_queue = Arc::new(JobQueue::default());

        if config.sync.use_git {
            sync(
                &location,
                config.sync.sync_interval,
                config.sync.branch.clone(),
                job_queue.clone(),
            )
            .await;
        }
        build_search_index(location.clone().into()).await;
        let links = ref_hub.links();
        let titles = ref_hub.titles();
        build_tags_and_links(&location, links.clone(), titles.clone()).await;
        let queue = job_queue.clone();
        tokio::spawn(process_tasks(queue, location.clone(), links.clone(), titles.clone()));
        server(config.general, (links, titles, job_queue.clone())).await
    }
}

fn print_version() {
    println!("tendril-wiki v{}", env!("CARGO_PKG_VERSION"));
}

fn print_help() {
    println!(
        "\nConfig file found at \x1b[38;5;47m{:#?}\x1b[0m\nInstall files found at \x1b[38;5;37m{:#?}\x1b[0m\n",
        get_config_location().0,
        get_data_dir_location());
    print!(
        "Usage: tendril [options]
        Options:
        -i, --init                   Initialize config file and install
        -b, --build                  Build all pages as HTML and output to ./public
        -v, --version                Print version.
        -h, --help                   Show this message.
        -u, --update                 Update the installation by copying over any new files or updating config.toml.

",
    );
}

// TODO: Maybe later add in multi-config, multi-folder stuff
// -c <path>, --config <path>   Use config at <path>
//
// Examples:

//   - Start wiki in the ~/work/wiki directory
//         $ tendril ~/work/wiki
//   - Start wiki at location specified in config file
//         $ tendril
//   - Start wiki in current folder with a custom config file
//         $ tendril . -c ./config.toml
