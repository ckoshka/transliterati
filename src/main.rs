use rayon::prelude::*;
use std::collections::{HashMap, HashSet, BTreeMap};
//use tokenizers::{Token};
use std::sync::{Arc, Mutex};
//mod sift4;

use strsim;
//Takes an iterable, element1, and element2. Replaces all instances of element1 with element2 in a non-destructive way by creating a new iterable.
//Uses strsim::generic_levenshtein to compare the two iterables and return usize (minimum edit distance)
fn test_replacement(
    iterable_original: &Vec<i32>,
    iterable_transformed: &Vec<i32>,
    element1: &i32,
    element2: &i32,
) -> f64 {
    //Replaces
    let mut new_iterable = iterable_transformed.clone();
    for el in new_iterable.iter_mut() {
        if el.clone() == element2.clone() {
            *el = element1.clone();
        }
    }
    let score = strsim::generic_levenshtein(&iterable_original.clone(), &new_iterable);
    return score as f64;
    //Cheaper option: compare the indices of element1 in iterable_original and the indices of element2 in iterable_transformed
}

//Takes two Vec<i32>, representing tokens - tokens_a and tokens_b. Creates an empty HashMap which will contain mappings from tokens in A to B. Creates two empty hash sets, tokens_a_set and tokens_b_set. In a loop (until all tokens from a and b are exhausted), it takes the "first" token from tokens_a_set and ranks it against all tokens in tokens_b_set via test_replacement. The lowest score is the best match. The mapping is added to the hashmap. Both tokens are removed from tokens_a_set and tokens_b_set.
//Returns a HashMap<i32, i32>
fn find_best_match(tokens_a: &Vec<i32>, tokens_b: &Vec<i32>) -> HashMap<i32, Vec<(i32, f64)>> {
    let mapping = Arc::new(Mutex::new(HashMap::new()));
    let mut tokens_a_set: HashSet<i32> = HashSet::new();
    let mut tokens_b_set: HashSet<i32> = HashSet::new();
    for token in tokens_a.iter() {
        tokens_a_set.insert(token.clone());
    }
    for token in tokens_b.iter() {
        tokens_b_set.insert(token.clone());
    }
    //Todo: since it's quadratic time, we could split the tokens_a and tokens_b into two halves and parallelize this
    tokens_a_set
        .clone()
        .par_iter()
        .for_each(|token_to_replace| {
            //for token in tokens_b_set.iter() {
            //let score = test_replacement(&tokens_a.clone(), &token_to_replace.clone(), &token.clone());
            //if score < best_score {
            //best_score = score;
            //best_token = token.clone();
            //}
            //}
            //Naive implementation, we can use par_sort instead:
            let mut sorted_tokens = tokens_b_set
                .par_iter()
                .map(|token| {
                    let score = test_replacement(
                        &tokens_a.clone(),
                        &tokens_b.clone(),
                        &token_to_replace.clone(),
                        &token.clone(),
                    );
                    (token.clone(), score / tokens_a.len() as f64)
                })
                .collect::<Vec<(i32, f64)>>();
            sorted_tokens.par_sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            //let best_token = sorted_tokens.first().unwrap().1;
            //Instead of returning just the top result, we could return everything and then merge the lists later.
            mapping
                .lock()
                .unwrap()
                .insert(token_to_replace.clone(), sorted_tokens);
        });
    return mapping.lock().unwrap().clone();
}

use tokenizers;
//use tokenizers::models::wordpiece::WordPiece;
//use tokenizers::models::wordpiece::WordPieceTrainer;
//use tokenizers::models::wordpiece::WordPieceBuilder;
use tokenizers::models::bpe::{BpeBuilder, BpeTrainer};
use tokenizers::tokenizer::Trainer;
use tokenizers::{AddedToken, Model, Tokenizer};

pub fn get_tokenizer_from_text(content: String, vocab_size: usize) -> Tokenizer {
    // Count words
    let mut word_counts = HashMap::new();
    for word in content
        .to_lowercase()
        .clone()
        .chars()
        .filter(|c| c.is_alphabetic() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
    {
        let spaced_word = format!("{}", word.trim());
        let count = word_counts.entry(spaced_word).or_insert(0 as u32);
        *count += 1;
    }

    let added_tokens = vec![
        AddedToken::from(String::from("[UNK]"), true),
        //AddedToken::from(String::from(" "), true),
    ];

    let mut trainer = BpeTrainer::builder()
        .show_progress(true)
        .vocab_size(vocab_size)
        .special_tokens(added_tokens.clone())
        .build();

    //Saves the string content to a file and then delet
    let mut model = BpeBuilder::default()
        //.files(filename.clone())
        .unk_token("[UNK]".to_string())
        //.continuing_subword_prefix("##".to_string())
        .build()
        .unwrap();

    let _result = trainer.feed(
        content
            .to_lowercase()
            .clone()
            .chars()
            .filter(|c| c.is_alphabetic())
            .map(|c| c.to_string())
            .collect::<Vec<String>>()
            .iter(),
        |s| Ok(vec![s.to_owned().to_string()]),
    );

    trainer.do_train(&word_counts, &mut model).unwrap();

    println!("Final vocab size: {:?}", model.get_vocab_size());
    println!("Preview: {:?}", model.get_vocab().iter().take(8).collect::<Vec<_>>());

    let mut tokenizer = Tokenizer::new(model);
    tokenizer.add_special_tokens(&added_tokens);

    tokenizer
}

pub fn encode_sentences(tokenizer: &Tokenizer, sentences: Vec<&str>) -> Vec<Vec<i32>> {
    let encodings = tokenizer.encode_batch(sentences, true).unwrap();
    let mut tokens = Vec::new();
    for encoding in encodings {
        tokens.push(
            encoding
                .get_ids()
                .iter()
                .map(|id| id.clone() as i32)
                .collect::<Vec<i32>>(),
        );
    }
    tokens
}

//use tokenizers::tokenizer::Decoder;
use serde_json::json;

fn main() {
    let file1txt = std::fs::read_to_string(
        std::env::args().nth(1).expect("The first argument should be a text file with different chunks separated by newlines, with no empty or very short lines."))
                .unwrap()
                .to_lowercase();
    let file2txt = std::fs::read_to_string(
        std::env::args().nth(2).expect("The second argument should be a text file with different chunks separated by newlines, with no empty or very short lines. It should correspond to the text in the first argument."))
                .unwrap()
                .to_lowercase();
    let mut number_of_unique_characters_1 = HashSet::new();
    for char in file1txt.to_lowercase().chars() {
        number_of_unique_characters_1.insert(char);
    }
    let mut number_of_unique_characters_2 = HashSet::new();
    for char in file2txt.to_lowercase().chars() {
        number_of_unique_characters_2.insert(char);
    }
    let mut max_vocab_size = std::cmp::max(
        number_of_unique_characters_1.len(),
        number_of_unique_characters_2.len(),
    );
    if let Some(arg) = std::env::args().nth(3) {
        max_vocab_size = arg.parse::<usize>().unwrap();
        println!("You have selected {:?} as your vocab size", max_vocab_size);
    } else {
        max_vocab_size = max_vocab_size * 1.18 as usize;
        println!("Remember, you can specify a vocab size as your third argument and test out different combinations. I have selected {} automatically", max_vocab_size);
    }
    let now = std::time::Instant::now();
    let tokenizer1 = get_tokenizer_from_text(file1txt.to_string(), max_vocab_size);
    let tokenizer2 = get_tokenizer_from_text(file2txt.to_string(), max_vocab_size);
    let tokens1 = encode_sentences(&tokenizer1, file1txt.split('\n').collect::<Vec<&str>>());
    let tokens2 = encode_sentences(&tokenizer2, file2txt.split('\n').collect::<Vec<&str>>());
    //let to_transliterate_tokens = tokens1.first().unwrap();
    //let mut to_transliterate = std::fs::read_to_string("data/to_translit").unwrap();
    //to_transliterate = to_transliterate.to_lowercase();
    //let new = to_transliterate.split_whitespace().map(
    //    |s| s.clone()
    //).collect::<Vec<&str>>();
    //let to_transliterate_tokens = encode_sentences(&tokenizer2, new.clone());
    let hashmaps_for_chunks = tokens1
        .par_iter()
        .zip(tokens2.par_iter())
        .map(|(token1, token2)| {
            let hashmap_for_chunk = find_best_match(&token1, &token2);
            hashmap_for_chunk
        })
        .collect::<Vec<HashMap<i32, Vec<(i32, f64)>>>>();
    //Merge the hashmaps. The i32 in each Vec represents a token_id, and the f64 is the score it received. We simply take the rank of each i32 token_id after being sorted.
    //It's not working because for different texts, different tokens are being covered. Some are being scored and some aren't.
    let mut merged_hashmap: HashMap<i32, HashMap<i32, f64>> = HashMap::new();
    let mut token_counts: HashMap<i32, u32> = HashMap::new();
    for hashmap in hashmaps_for_chunks.iter() {
        for (token_id, scores) in hashmap.iter() {
            let merged_scores = merged_hashmap
                .entry(token_id.clone())
                .or_insert(HashMap::new());
            let mut sorted_scores = scores.clone();
            sorted_scores.par_sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            let positions = sorted_scores.iter().enumerate().take(10);
            for (i, (token_id, _)) in positions {
                if let Some(current_score) = merged_scores.get_mut(&token_id) {
                    let number_of_existing_entries =
                        token_counts.entry(token_id.clone()).or_insert(0 as u32);
                    //If there's one already, it should be 1/2 and 1/2. If there's two, the old score should be weighted as 2/3 and the new score as 1/3. number_of_existing_entries/number_of_existing_entries + 1. and 1 / number_of_existing_entries + 1.
                    *current_score =
                        (((current_score.sqrt() * number_of_existing_entries.clone() as f64) + i as f64)
                            / (number_of_existing_entries.clone() as f64 + 1.0) as f64)
                            .powi(2);
                    *number_of_existing_entries += 1;
                } else {
                    merged_scores.insert(token_id.clone(), i.clone().pow(2) as f64);
                    *token_counts.entry(token_id.clone()).or_insert(0 as u32) += 1;
                }
            }
        }
    }
    //Now we sort each hashmap by the f64 values, taking the lowest 5 results along with their "likelihood"
    //uses tokenizer1.id_to_token(token_id.clone() as u32).unwrap(), for each token_id in the top 5
    println!("Estimating probabilities!");
    let mut sorted_merged_hashmap: HashMap<String, HashMap<usize, _>> = HashMap::new();
    for (token_id, scores) in merged_hashmap.iter() {
        let mut sorted_scores = scores
            .par_iter()
            .map(|(token_id, score)| (token_id.clone(), *score))
            .collect::<Vec<(i32, f64)>>();
        sorted_scores.par_sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        let best_token_ids = sorted_scores.iter().take(3);
        sorted_merged_hashmap.insert(
            tokenizer1.id_to_token(token_id.clone() as u32).unwrap(),
            HashMap::from_iter(
                best_token_ids
                    .enumerate()
                    .map(|(indx, (token_id, _))| {
                        (
                            indx, tokenizer2.id_to_token(token_id.clone() as u32).unwrap()
                        )
                    })
                    .collect::<Vec<(usize, _)>>()
                    .into_iter(),
            ),
        );
    }
    let final_table = json!(sorted_merged_hashmap);
    //pretty-print the json
    let pretty_json = serde_json::to_string_pretty(&final_table).unwrap();
    println!("{}", pretty_json);
    println!("Took {:?} milliseconds", now.elapsed().as_millis());
}
//TODO: make having probabilities optional
