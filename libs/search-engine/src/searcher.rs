use std::{collections::HashMap, path::PathBuf};

use tokio::fs::read_to_string;

use crate::{tokenizer::tokenize, Doc, Tokens};

fn tokenize_query(query: &str) -> Tokens {
    tokenize(query)
}

pub(crate) async fn search(query: &str, index_location: PathBuf) -> Vec<Doc> {
    let tokens = tokenize_query(query);
    let keys = tokens.keys();
    // let now = Instant::now();
    let search_idx = read_to_string(index_location.join("search-index.json"))
        .await
        .unwrap();
    let search_idx: HashMap<String, Vec<String>> = serde_json::from_str(&search_idx).unwrap();
    let mut relevant_docs = Vec::<Doc>::new();
    let doc_idx = read_to_string(index_location.join("docs.json"))
        .await
        .unwrap();
    let mut doc_idx: HashMap<String, Doc> = serde_json::from_str(&doc_idx).unwrap();
    // println!("serializing took: {:?}", now.elapsed());
    // tokenized query terms + the number of possible variations
    let mut tokens_for_scoring = Vec::with_capacity(keys.len() * 17);
    keys.for_each(|key| {
        let variations = variations_of_word(key);
        for variation in variations {
            if let Some(doc_ids) = search_idx.get(&variation) {
                tokens_for_scoring.push(variation);
                for doc_id in doc_ids {
                    if let Some(doc_body) = doc_idx.remove(doc_id) {
                        relevant_docs.push(doc_body);
                    }
                }
            }
        }
    });
    rank_docs(relevant_docs, tokens_for_scoring, doc_idx.keys().len())
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
fn rank_docs(mut relevant_docs: Vec<Doc>, tokens: Vec<String>, total_docs: usize) -> Vec<Doc> {
    let inverse_document_frequency =
        (total_docs as f32 / relevant_docs.len() as f32 + 1.0).ln() + 1.0;

    relevant_docs.sort_by(|a, b| {
        let processed_a = term_frequency(a, &tokens) * inverse_document_frequency;
        let processed_b = term_frequency(b, &tokens) * inverse_document_frequency;
        processed_b.partial_cmp(&processed_a).unwrap()
    });
    relevant_docs
}

/// This is a sum of the frequency of N number of tokens in a document.
fn term_frequency(doc: &Doc, tokens: &[String]) -> f32 {
    tokens.iter().fold(0.0, |score, tok| {
        if let Some(doc_score) = doc.tokens.get(tok) {
            // term frequency adjusted for document length
            *doc_score as f32 / doc.tokens.len() as f32
        } else {
            score
        }
    })
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
