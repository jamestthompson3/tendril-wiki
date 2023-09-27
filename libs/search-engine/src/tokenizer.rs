use std::collections::HashMap;

use regex::Regex;

const STOP_WORDS: [&str; 51] = [
    "a", "about", "an", "are", "and", "as", "at", "be", "but", "by", "co", "com", "do", "don't",
    "for", "from", "has", "have", "he", "his", "her", "hers", "http", "https", "i", "i'm", "in",
    "is", "it", "it's", "just", "like", "me", "my", "not", "of", "on", "or", "so", "that", "the",
    "they", "this", "to", "was", "www", "we", "were", "with", "you", "your",
];

lazy_static::lazy_static! {
    // |#|%|}|\*|<|>|_ might also need this.
    pub(crate) static ref PUNCT_RGX: Regex = Regex::new(r"[[[:punct:]]]").unwrap();
    static ref STOP_WORD_MAP: HashMap<&'static str, bool> = {
   let mut m = HashMap::with_capacity(51);
    for word in STOP_WORDS {
        m.insert(word, true);
        }
    m
    };
}

pub(crate) fn tokenize(slice: &str) -> Vec<String> {
    let punct_to_whitespace = PUNCT_RGX.replace_all(slice, " ");
    punct_to_whitespace
        .split(' ')
        .map(|w| {
            let word = w.to_lowercase();
            word.replace('\n', "")
        })
        .filter(|w| STOP_WORD_MAP.get(w.as_str()).is_none() && !w.is_empty() && w.len() <= 80)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_strings() {
        let test_string =
            "A tested, tokenized string. This could represent a note, a bookmark, or an idea.";
        let tokens = [
            "tested",
            "tokenized",
            "string",
            "could",
            "represent",
            "note",
            "bookmark",
            "idea",
        ];
        let tokenized = tokenize(test_string);
        assert_eq!(tokenized, tokens);
    }

    #[test]
    fn tokenizes_strings_with_links() {
        let test_string = "[[link|https://teukka.tech/luanvim.html]]";
        let tokens = ["link", "teukka", "tech", "luanvim", "html"];
        let tokenized = tokenize(test_string);
        assert_eq!(tokenized, tokens);
    }
}
