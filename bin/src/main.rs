use build::{
    build_tags_and_links, config::read_config, delete_from_global_store, get_config_location,
    get_data_dir_location, install, pages::Builder, rename_in_global_store, update,
    update_global_store, update_mru_cache, RefHub,
};
use futures::{stream, StreamExt};
use persistance::fs::{
    create_journal_entry, get_file_path, normalize_wiki_location, path_to_data_structure,
    write_archive,
};
use search_engine::{
    build_search_index, delete_archived_file, delete_entry_from_update, patch_search_from_archive,
    patch_search_from_update,
};
use std::{
    path::PathBuf,
    process::exit,
    sync::Arc,
    time::{Duration, Instant},
};
use tasks::{archive::{extract, compress}, git_update, messages::Message, sync, JobQueue, Queue};
use tokio::{fs, time::sleep};
use www::server;

const NUM_JOBS: u32 = 50;

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
            _ => {
                if arg.starts_with('-') {
                    eprintln!("unknown option: {}", arg);
                    exit(1);
                }
                if !arg.is_empty() {
                    let config = read_config();
                    let location = normalize_wiki_location(&config.general.wiki_location);
                    create_journal_entry(&location, args.join(" "))
                        .await
                        .unwrap();
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
        build_tags_and_links(&location, links.clone()).await;
        let queue = job_queue.clone();
        tokio::spawn(async move {
            loop {
                let jobs = match queue.pull(NUM_JOBS).await {
                    Ok(jobs) => jobs,
                    Err(err) => {
                        eprintln!("{}", err);
                        panic!("Failed to pull jobs");
                    }
                };
                stream::iter(jobs)
                    .for_each_concurrent(NUM_JOBS as usize, |job| async {
                        match job.message {
                            Message::Rebuild => {
                                build_tags_and_links(&location, links.clone()).await;
                            }
                            Message::Patch { patch } => {
                                let note = patch.clone().into();

                                update_global_store(&patch.title, &note, links.clone()).await;
                                patch_search_from_update(&note).await;

                                if !patch.old_title.is_empty() && patch.old_title != patch.title {
                                    rename_in_global_store(
                                        &patch.title,
                                        &patch.old_title,
                                        &location,
                                        links.clone(),
                                    )
                                    .await;
                                }
                                update_mru_cache(&patch.old_title, &patch.title).await;
                            }
                            Message::Delete { title } => {
                                let path = get_file_path(&location, &title).unwrap_or_else(|_| {
                                    panic!("Failed to find file for deletion: {}", title)
                                });
                                let note = path_to_data_structure(&path).await.unwrap();
                                persistance::fs::delete(
                                    &location,
                                    &title,
                                )
                                .await
                                .unwrap();
                                delete_from_global_store(&title, &note, links.clone()).await;
                                delete_entry_from_update(&title).await;
                                delete_archived_file(&title).await;
                            }
                            Message::Archive { url, title } => {
                                let text =
                                    tokio::task::spawn_blocking(|| extract(url)).await.unwrap();
                                let compressed = compress(&text);
                                write_archive(compressed, &title).await;
                                patch_search_from_archive((title, text)).await;
                            }
                        }
                    })
                    .await;
                sleep(Duration::from_millis(10)).await;
            }
        });
        server(config.general, (ref_hub.links(), job_queue.clone())).await
    }
}

fn print_version() {
    println!("tendril-wiki v{}", env!("CARGO_PKG_VERSION"),);
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
