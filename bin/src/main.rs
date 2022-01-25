use build::{
    build_tags_and_links, config::read_config, create_journal_entry, delete_from_global_store,
    get_config_location, get_data_dir_location, install, pages::Builder, update,
    update_global_store, RefHub, RefHubRx, RefHubTx,
};
use std::{path::PathBuf, process::exit, time::Instant};
use tasks::{git_update, normalize_wiki_location, sync};
use tokio::sync::mpsc;
use www::server;

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
                    create_journal_entry(location.clone(), args.join(" ")).unwrap();
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
            std::fs::remove_dir_all("./public").unwrap();
        }
        let builder = Builder::new();
        builder.sweep(&location);
        builder.compile_all();
        println!("Built static site in: {}ms", now.elapsed().as_millis());
    } else {
        let ref_hub = RefHub::new();
        let (tx, mut rx): (RefHubTx, RefHubRx) = mpsc::channel(50);
        let watcher_links = ref_hub.links();

        if config.sync.use_git {
            sync(
                &location,
                config.sync.sync_interval,
                config.sync.branch.clone(),
                tx.clone(),
            )
            .await;
        }
        tokio::spawn(async move {
            while let Some((cmd, file)) = rx.recv().await {
                match cmd.as_ref() {
                    "rebuild" => {
                        build_tags_and_links(&location, watcher_links.clone()).await;
                    }
                    "update" => {
                        update_global_store(&file, &location, watcher_links.clone());
                    }
                    "delete" => {
                        // TODO: figure out why making this async causes the tokio::spawn call to
                        // give compiler errors.
                        delete_from_global_store(&file, &location, watcher_links.clone());
                    }
                    _ => {}
                }
            }
        });
        tx.send(("rebuild".into(), "".into())).await.unwrap();
        server(config.general, (ref_hub.links(), tx.clone())).await
    }
}

fn print_version() {
    println!("tendril-wiki v{}", env!("CARGO_PKG_VERSION"),);
}

fn print_help() {
    println!(
        "\nConfig file found at {}\nInstall files found at {}\n",
        format!("\x1b[38;5;47m{:#?}\x1b[0m", get_config_location().0),
        format!("\x1b[38;5;37m{:#?}\x1b[0m", get_data_dir_location())
    );
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
