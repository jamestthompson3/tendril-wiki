use std::collections::HashMap;

use crate::{read_search_index, tokenizer::tokenize, SearchIndexReadErr};

fn tokenize_query(query: &str) -> Vec<String> {
    tokenize(query)
}

pub(crate) async fn search(query: &str) -> Vec<String> {
    let tokens = tokenize_query(query);

    let mut results = Vec::<(String, f32)>::new();
    let mut document_appearences: HashMap<String, usize> = HashMap::new();
    tokens.iter().for_each(|key| {
        let variations = variations_of_word(key);
        for variation in variations {
            match read_search_index(&variation) {
                Ok(entries) => {
                    for entry in entries {
                        if let Some(count) = document_appearences.get_mut(&entry.0) {
                            *count += 1;
                            continue;
                        }
                        document_appearences.insert(entry.0.to_string(), 1);
                        results.push(entry)
                    }
                }
                Err(e) => match e {
                    SearchIndexReadErr::NotExistErr => {
                        continue;
                    }
                    SearchIndexReadErr::DeserErr(e) => {
                        eprintln!("Could not deserialize: {}", e);
                    }
                },
            }
        }
    });
    // TODO: Rank based on number of ties a word occurs in a doc, discounted for doc length
    //       Titles should weigh heavier.
    //       Maybe some sort of proximity ranking?
    rank_docs(&document_appearences, results, query)
}

/// use term frequency-inverse document frequency to rank the search results.
/// We use term frequency adjusted for document length accumulated over all tokens in the search
/// query
/// We use the inverse document frequency smooth weight (log(N / 1 + nt) + 1)
///
/// ### What is a document in this context?
///
/// A document is a `Doc` data structure which can be derived from multiple sources (though at the
/// moment it is only derived from wiki notes).
fn rank_docs(
    doc_frequency: &HashMap<String, usize>,
    mut results: Vec<(String, f32)>,
    query: &str,
) -> Vec<String> {
    results.sort_by(|a, b| {
        let mut processed_a = a.1 * *doc_frequency.get(&a.0).unwrap() as f32;
        if a.0.contains(query) {
            processed_a *= 2.0;
        }
        let mut processed_b = b.1 * *doc_frequency.get(&b.0).unwrap() as f32;
        if b.0.contains(query) {
            processed_b *= 2.0;
        }
        processed_b.partial_cmp(&processed_a).unwrap()
    });
    results.iter().map(|r| r.0.to_owned()).collect()
}

fn variations_of_word(key: &str) -> Vec<String> {
    let word_stem = stem::get(key).unwrap();
    let mut variations = Vec::with_capacity(19);
    // Very very hacky lemmatization
    for ending in WORD_ENDINGS {
        variations.push(format!("{}{}", word_stem, ending));
    }
    variations.push(key.into());
    variations.push(word_stem);
    variations
}

const WORD_ENDINGS: [&str; 17] = [
    "e", "s", "ly", "ment", "ed", "'s", "or", "er", "ing", "y", "tion", "ies", "r", "ation", "d",
    "n", "ian",
];

const OPEN_TAG_LENGTH: usize = 6;
const CLOSE_TAG_LENGTH: usize = 7;

pub(crate) fn highlight_matches(mut line: String, term: &str) -> String {
    let readline = line.clone().to_lowercase();
    let matches = readline
        .match_indices(&term.trim().to_lowercase())
        .collect::<Vec<(usize, &str)>>();
    if !matches.is_empty() {
        for (pointer, (idx, t)) in matches.into_iter().enumerate() {
            let current_pos = idx + (pointer * (OPEN_TAG_LENGTH + CLOSE_TAG_LENGTH));
            let closing_tag = current_pos + OPEN_TAG_LENGTH + t.len();
            line.insert_str(current_pos, "<mark>");
            line.insert_str(closing_tag, "</mark>");
        }
    }
    line
}
