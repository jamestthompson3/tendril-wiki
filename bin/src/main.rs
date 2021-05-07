
use build::{RefBuilder, config::read_config, pages::Builder, print_config_location};
use tasks::{search, sync};
use www::server;
use std::process::exit;

#[tokio::main]
async fn main() {
    let args = std::env::args().skip(1).collect::<Vec<String>>();
    let mut build_all = false;
    for arg in args.iter() {
        match arg.as_ref() {
            "-v" | "--version" => return print_version(),
            "-h" | "--help" => return print_help(),
            "-b" | "--build" => build_all = true,
            "-c" | "--config" => return print_config_location(),
            _ => {
                if arg.starts_with('-') {
                    eprintln!("unknown option: {}", arg);
                    exit(1);
                }
            }
        }
    };
    let config = read_config();
    if build_all {
        std::fs::remove_dir_all("./public").unwrap();
        let builder = Builder::new();
        builder.sweep(&config.general.wiki_location);
        builder.compile_all();
    } else {
        if config.sync.use_git {
            sync(&config.general.wiki_location, config.sync.sync_interval, config.sync.branch);
        }
        let mut ref_builder = RefBuilder::new();
        ref_builder.build(&config.general.wiki_location);
        server(config.general, ref_builder).await;
    }
}


fn print_version() {
    println!("tendril-wiki v{}", env!("CARGO_PKG_VERSION"))
}
fn print_help() {
    print!(
        "Usage: tendril [options]
        Options:
        -b, --build    Build all pages as HTML and output to ./public
        -v, --version  Print version.
        -h, --help     Show this message.
        -c, --config   Show the config file location
        ",
        );
}
