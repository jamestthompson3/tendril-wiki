// NOTES:
// keep a arc hashmap to track backlinks while calling to_html
// {'filename': ['referrer1', 'referrer2']}
//

mod build_pages;
mod config;

use build_pages::build;
use config::read_config;
use www::server;

#[tokio::main]
async fn main() {
    let config = read_config();
    build(&config.wiki_location);
    server(config.port).await;
}
