use std::{collections::HashMap, path::PathBuf};

use persistance::fs::{path_to_string, utils::get_file_path};

use crate::{read_search_index, tokenizer::tokenize, Doc, SearchIndexReadErr};

fn tokenize_query(query: &str) -> Vec<String> {
    tokenize(query)
}

pub(crate) async fn search(query: &str) -> Vec<String> {
    let tokens = tokenize_query(query);

    let mut results = Vec::<String>::new();
    let mut read_files: HashMap<String, Vec<String>> = HashMap::new();
    let mut file_term_count: HashMap<String, usize> = HashMap::new();
    tokens.iter().for_each(|key| {
        let variations = variations_of_word(key);
        for variation in variations {
            match read_search_index(&variation) {
                Ok(entries) => {
                    for entry in entries {
                        if read_files.get(&entry.0).is_some() {
                            file_term_count
                                .entry(entry.0.to_owned())
                                .and_modify(|v| *v += 1);
                            continue;
                        }
                        let file_path = get_file_path(&entry.0).unwrap();
                        let content = path_to_string(&file_path).unwrap();
                        let lines = content.lines().collect::<Vec<&str>>();
                        read_files.insert(
                            entry.0.to_string(),
                            lines.iter().map(|l| l.to_string()).collect(),
                        );
                        file_term_count.entry(entry.0.to_owned()).or_insert(1);
                        results.push(entry.0.to_owned())
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
    rank_docs(&read_files, results, &file_term_count, query)
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
    read_files: &HashMap<String, Vec<String>>,
    mut results: Vec<String>,
    term_counts: &HashMap<String, usize>,
    query: &str,
) -> Vec<String> {
    results.sort_by(|a, b| {
        let num_appearances_a = term_counts.get(a.as_str()).unwrap();
        let text_len_a = read_files.get(a.as_str()).unwrap().len();
        let num_appearances_b = term_counts.get(b.as_str()).unwrap();
        let text_len_b = read_files.get(b.as_str()).unwrap().len();
        let mut processed_a = term_frequency(*num_appearances_a, text_len_a);
        if a.as_str().contains(query) {
            processed_a *= 2.0;
        }
        let mut processed_b = term_frequency(*num_appearances_b, text_len_b);
        if b.as_str().contains(query) {
            processed_b *= 2.0;
        }

        processed_b.partial_cmp(&processed_a).unwrap()
    });
    results
}

/// This is a sum of the frequency of N number of tokens in a document.
fn term_frequency(num_appearances: usize, text_len: usize) -> f32 {
    num_appearances as f32 / text_len as f32
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
