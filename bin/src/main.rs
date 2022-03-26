use build::{
    build_tags_and_links, config::read_config, create_journal_entry, delete_from_global_store,
    get_config_location, get_data_dir_location, install, pages::Builder, purge_file,
    rename_in_global_store, update, update_global_store, update_mru_cache, RefHub, RefHubRx,
    RefHubTx,
};
use std::{path::PathBuf, process::exit, time::Instant};
use tasks::{git_update, normalize_wiki_location, sync};
use tokio::{fs, sync::mpsc};
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
        let (tx, mut rx): (RefHubTx, RefHubRx) = mpsc::channel(50);

        if config.sync.use_git {
            sync(
                &location,
                config.sync.sync_interval,
                config.sync.branch.clone(),
                tx.clone(),
            )
            .await;
        }
        let watcher_links = ref_hub.links();
        tokio::spawn(async move {
            while let Some((cmd, file)) = rx.recv().await {
                match cmd.as_ref() {
                    "rebuild" => {
                        build_tags_and_links(&location, watcher_links.clone()).await;
                    }
                    "update" => {
                        if let [old_title, current_title] =
                            file.split("~~").collect::<Vec<&str>>()[..]
                        {
                            update_global_store(current_title, &location, watcher_links.clone())
                                .await;

                            if !old_title.is_empty() && old_title != current_title {
                                rename_in_global_store(
                                    current_title,
                                    old_title,
                                    &location,
                                    watcher_links.clone(),
                                )
                                .await;
                            }
                            update_mru_cache(old_title, current_title).await;
                        }
                    }
                    "delete" => {
                        // TODO: figure out why making this async causes the tokio::spawn call to
                        // give compiler errors.
                        delete_from_global_store(&file, &location, watcher_links.clone()).await;
                        purge_file(&location, &file).await
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
