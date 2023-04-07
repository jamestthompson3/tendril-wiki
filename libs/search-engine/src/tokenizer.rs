use std::{collections::HashMap, usize};

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

pub(crate) fn tokenize(slice: &str) -> HashMap<String, usize> {
    let stripped_whitespace = PUNCT_RGX.replace_all(slice, " ");
    stripped_whitespace
        .split(' ')
        .map(|w| {
            let word = w.to_lowercase();
            word.replace('\n', "")
        })
        .filter(|w| STOP_WORD_MAP.get(w.as_str()).is_none() && !w.is_empty())
        .fold(HashMap::new(), |mut map, token| {
            map.entry(token).and_modify(|v| *v += 1).or_insert(1);
            map
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_strings() {
        let test_string =
            "A tested, tokenized string. This could represent a note, a bookmark, or an idea.";
        let mut mapped = HashMap::new();
        mapped.insert("tested".to_owned(), 1);
        mapped.insert("tokenized".to_owned(), 1);
        mapped.insert("string".to_owned(), 1);
        mapped.insert("represent".to_owned(), 1);
        mapped.insert("note".to_owned(), 1);
        mapped.insert("bookmark".to_owned(), 1);
        mapped.insert("could".to_owned(), 1);
        mapped.insert("idea".to_owned(), 1);
        let tokenized = tokenize(test_string);
        assert_eq!(tokenized, mapped);
    }

    #[test]
    fn tokenizes_strings_with_links() {
        let test_string = "[[link|https://teukka.tech/luanvim.html]]";
        let mut mapped = HashMap::new();
        mapped.insert("teukka".to_owned(), 1);
        mapped.insert("tech".to_owned(), 1);
        mapped.insert("luanvim".to_owned(), 1);
        mapped.insert("html".to_owned(), 1);
        mapped.insert("link".to_owned(), 1);
        let tokenized = tokenize(test_string);
        assert_eq!(tokenized, mapped);
    }
}
