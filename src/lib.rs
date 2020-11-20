#[macro_use]
extern crate cached;

use crate::ngram::from_ser;
use trie::Trie;

use crate::crossword::Crossword;

use crate::crossword::Direction;
use crate::ngram::bigrams;
use fill::parallel::fill_crossword as fill;
use std::fs::File;
use std::sync::Arc;
use std::{collections::HashMap, time::Instant};

pub mod crossword;
pub mod fill;
mod ngram;
mod order;
mod parse;
pub mod trie;

pub fn fill_crossword(contents: String, words: Vec<String>) -> Result<Crossword, String> {
    let crossword = Crossword::new(contents).unwrap();
    let (bigrams, trie) = index_words(words);
    fill(&crossword, Arc::new(trie), Arc::new(bigrams))
}

pub fn default_indexes() -> (HashMap<(char, char), usize>, Trie) {
    let now = Instant::now();
    let file = File::open("./trie.bincode").unwrap();
    let load = bincode::deserialize_from::<File, Trie>(file);
    let trie = load.unwrap();
    println!("Loaded trie in {}ms", now.elapsed().as_millis());
    let now = Instant::now();

    let file = File::open("./bigrams.bincode").unwrap();
    let load = bincode::deserialize_from::<File, HashMap<String, usize>>(file);
    let bigrams = from_ser(load.unwrap());
    println!("Loaded bigrams in {}ms", now.elapsed().as_millis());

    (bigrams, trie)
}

pub fn default_words() -> Vec<String> {
    let file = File::open("wordlist.json").unwrap();
    serde_json::from_reader(file).expect("JSON was not well-formatted")
}

pub fn index_words(raw_data: Vec<String>) -> (HashMap<(char, char), usize>, Trie) {
    let bigram = bigrams(&raw_data);
    let trie = Trie::build(raw_data);
    (bigram, trie)
}

#[cfg(test)]
mod tests {
    use crate::ngram::from_ser;
    use std::{collections::HashMap, time::Instant};

    use crate::index_words;
    use crate::ngram::to_ser;
    use crate::File;
    use crate::{default_words, trie::Trie};

    #[test]
    #[ignore]
    fn rebuild_serialized_indexes() {
        let (bigrams, trie) = index_words(default_words());

        let trie_file = File::create("trie.bincode").unwrap();
        let trie_result = bincode::serialize_into(trie_file, &trie);
        assert!(trie_result.is_ok());

        let bigrams_file = File::create("bigrams.bincode").unwrap();
        let bigrams_result = bincode::serialize_into(bigrams_file, &to_ser(bigrams));
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
        let load = bincode::deserialize_from::<File, HashMap<String, usize>>(file);
        assert!(load.is_ok());
        from_ser(load.unwrap());
    }
}
