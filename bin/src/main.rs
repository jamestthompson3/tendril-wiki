
use build::{RefBuilder, pages::Builder, config::read_config};
use www::server;

#[tokio::main]
async fn main() {
    let args = std::env::args().skip(1).collect::<Vec<String>>();
    let mut args = args.iter();
    let mut build_all = false;
    while let Some(arg) = args.next() {
        match arg.as_ref() {
            "-v" | "-version" | "--version" => return print_version(),
            "-h" | "-help" | "--help" => return print_help(),
            "-b" | "-build" | "--build" => build_all = true,
            _ => {
                if arg.starts_with('-') {
                    return eprintln!("unknown option: {}", arg);
                }
            }
        }
    };
    let config = read_config();
    if build_all {
        std::fs::remove_dir_all("./public").unwrap();
        let builder = Builder::new();
        builder.sweep(&config.wiki_location);
        builder.compile_all();
    } else {
        build::sync::check_sync(&config.wiki_location);
        let mut ref_builder = RefBuilder::new();
        ref_builder.build(&config.wiki_location);
        server(config.clone(), ref_builder).await;
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
        ",
        );
}
