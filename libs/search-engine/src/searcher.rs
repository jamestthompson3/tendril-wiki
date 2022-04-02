use std::{collections::HashMap, time::Instant};

use tokio::fs::read_to_string;

use crate::{tokenizer::tokenize, Doc, Tokens};

fn tokenize_query(query: &str) -> Tokens {
    tokenize(query)
}

pub(crate) async fn search<'a>(query: &str) -> Vec<Doc<'a>> {
    let tokens = tokenize_query(query);
    let keys = tokens.keys();
    let now = Instant::now();
    let search_idx = read_to_string("./search-index.json").await.unwrap();
    let search_idx: HashMap<String, Vec<String>> = serde_json::from_str(&search_idx).unwrap();
    let mut relevant_docs = Vec::<Doc>::new();
    let doc_idx = read_to_string("./docs.json").await.unwrap();
    let mut doc_idx: HashMap<String, Doc> = serde_json::from_str(&doc_idx).unwrap();
    println!("serializing took: {:?}", now.elapsed());
    keys.for_each(|key| {
        let variations = variations_of_word(key);
        for variation in variations {
            if let Some(doc_ids) = search_idx.get(&variation) {
                for doc_id in doc_ids {
                    if let Some(doc_body) = doc_idx.remove(doc_id) {
                        relevant_docs.push(doc_body);
                    }
                }
            }
        }
    });
    rank_docs(relevant_docs, tokens.keys().collect::<Vec<&String>>())
}

fn rank_docs<'a>(mut relevant_docs: Vec<Doc<'a>>, tokens: Vec<&String>) -> Vec<Doc<'a>> {
    let multiplier = (relevant_docs.len() / tokens.len()) as f32;
    relevant_docs.sort_by(|a, b| {
        let processed_a = fold_tokens_by_doc(a, &tokens) * multiplier;
        let processed_b = fold_tokens_by_doc(b, &tokens) * multiplier;
        processed_a.partial_cmp(&processed_b).unwrap()
    });
    relevant_docs
}

fn fold_tokens_by_doc(doc: &Doc, tokens: &[&String]) -> f32 {
    tokens.iter().fold(0.0, |score, &tok| {
        if let Some(doc_score) = doc.tokens.get(tok) {
            score - *doc_score as f32
        } else {
            score
        }
    })
}

fn variations_of_word(key: &str) -> Vec<String> {
    vec![String::from(key)]
}
