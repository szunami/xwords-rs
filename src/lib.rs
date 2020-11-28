extern crate rustc_hash;

use crate::fill::Filler;
use fill::single_threaded::SingleThreadedFiller;
use rustc_hash::FxHashMap;
use trie::Trie;

use crate::crossword::Crossword;

use crate::{crossword::Direction, ngram::bigrams};
use std::{fs::File, time::Instant};

pub mod crossword;
pub mod fill;
mod ngram;
pub mod order;
pub mod parse;
pub mod trie;

pub fn fill_crossword_with_default_wordlist(crossword: &Crossword) -> Result<Crossword, String> {
    let (bigrams, trie) = default_indexes();
    SingleThreadedFiller::new(&trie, &bigrams).fill(crossword)
}

pub fn default_indexes() -> (FxHashMap<(char, char), usize>, Trie) {
    let now = Instant::now();
    let file = File::open("./trie.bincode").unwrap();
    let load = bincode::deserialize_from::<File, Trie>(file);
    let trie = load.unwrap();
    println!("Loaded trie in {}ms", now.elapsed().as_millis());
    let now = Instant::now();

    let file = File::open("./bigrams.bincode").unwrap();
    let load = bincode::deserialize_from::<File, FxHashMap<(char, char), usize>>(file);
    let bigrams = load.unwrap();
    println!("Loaded bigrams in {}ms", now.elapsed().as_millis());

    (bigrams, trie)
}

pub fn default_words() -> Vec<String> {
    let file = File::open("wordlist.json").unwrap();
    serde_json::from_reader(file).expect("JSON was not well-formatted")
}

pub fn index_words(raw_data: Vec<String>) -> (FxHashMap<(char, char), usize>, Trie) {
    let bigram = bigrams(&raw_data);
    let trie = Trie::build(raw_data);
    (bigram, trie)
}

#[cfg(test)]
mod tests {
    use crate::FxHashMap;
    use std::time::Instant;

    use crate::{default_words, index_words, trie::Trie, File};

    #[test]
    #[ignore]
    fn rebuild_serialized_indexes() {
        let (bigrams, trie) = index_words(default_words());

        let trie_file = File::create("trie.bincode").unwrap();
        let trie_result = bincode::serialize_into(trie_file, &trie);
        assert!(trie_result.is_ok());

        let bigrams_file = File::create("bigrams.bincode").unwrap();
        let bigrams_result = bincode::serialize_into(bigrams_file, &bigrams);
        assert!(bigrams_result.is_ok());
    }

    #[test]
    fn test_trie_load() {
        let now = Instant::now();
        let file = File::open("./trie.bincode").unwrap();
        let load = bincode::deserialize_from::<File, Trie>(file);
        assert!(load.is_ok());
        println!("Loaded trie in {}", now.elapsed().as_millis());
    }

    #[test]
    fn test_bigrams_load() {
        let file = File::open("bigrams.bincode").unwrap();
        let load = bincode::deserialize_from::<File, FxHashMap<(char, char), usize>>(file);
        assert!(load.is_ok());
    }
}
