use compression::prelude::*;
use readability::extractor::{self, Product};

pub fn extract(url: String) -> Product {
    match extractor::scrape(&url) {
        Ok(product) => product,
        Err(e) => panic!("{}", e),
    }
}

pub fn compress(text: &str) -> Vec<u8> {
    text.as_bytes()
        .iter()
        .cloned()
        .encode(&mut BZip2Encoder::new(9), Action::Finish)
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
}
