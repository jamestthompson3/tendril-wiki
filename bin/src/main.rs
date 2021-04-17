mod build_pages;
mod config;

use build_pages::Builder;
use config::read_config;
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
    let builder = Builder::new();
    builder.sweep(&config.wiki_location);
    if build_all {
        builder.compile_all();
    } else {
        server(config.port).await;
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
