use readability::extractor;

pub fn extract(url: String) -> String {
    match extractor::scrape(&url) {
        Ok(product) => product.text,
        Err(e) => panic!("{}", e),
    }
}
