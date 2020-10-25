#[macro_use]
extern crate cached;

use crate::ngram::from_ser;
use trie::Trie;

use crate::crossword::Crossword;

use crate::crossword::Direction;
use crate::{ngram::bigrams, order::score_word};
use std::collections::HashMap;
use std::sync::Arc;

use std::{fmt, fs::File};

pub mod crossword;
pub mod fill;
mod ngram;
mod order;
mod parse;
pub mod trie;

pub fn fill_crossword(contents: String, words: Vec<String>) -> Result<Crossword, String> {
    let crossword = Crossword::new(contents).unwrap();
    let (bigrams, trie) = index_words(words);
    fill::fill_crossword(&crossword, Arc::new(trie), Arc::new(bigrams))
}

pub fn default_indexes() -> (HashMap<(char, char), usize>, Trie) {
    let file = File::open("./trie.json").unwrap();
    let load = serde_json::from_reader::<File, Trie>(file);
    let trie = load.unwrap();
    let file = File::open("./bigrams.json").unwrap();
    let load = serde_json::from_reader::<File, HashMap<String, usize>>(file);
    let bigrams = from_ser(load.unwrap());
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

// TODO: Remove this
#[derive(Debug, PartialEq, Clone)]
pub struct Word {
    contents: String,
    start_row: usize,
    start_col: usize,
    length: usize,
    direction: Direction,
}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "Contents: {}", self.contents)
    }
}

impl Word {
    pub fn new(
        contents: String,
        start_row: usize,
        start_col: usize,
        length: usize,
        direction: Direction,
    ) -> Word {
        Word {
            contents,
            start_row,
            start_col,
            length,
            direction,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ngram::from_ser;
    use std::collections::HashMap;

    use crate::index_words;
    use crate::ngram::to_ser;
    use crate::File;
    use crate::{default_words, trie::Trie};

    #[test]
    #[ignore]
    fn rebuild_serialized_indexes() {
        let (bigrams, trie) = index_words(default_words());

        let trie_file = File::create("trie.json").unwrap();
        let trie_result = serde_json::to_writer(trie_file, &trie);
        assert!(trie_result.is_ok());

        let bigrams_file = File::create("bigrams.json").unwrap();
        let bigrams_result = serde_json::to_writer(bigrams_file, &to_ser(bigrams));
        assert!(bigrams_result.is_ok());
    }

    #[test]
    fn test_trie_load() {
        let file = File::open("./trie.json").unwrap();
        let load = serde_json::from_reader::<File, Trie>(file);
        assert!(load.is_ok());
    }

    #[test]
    fn test_bigrams_load() {
        let file = File::open("./bigrams.json").unwrap();
        let load = serde_json::from_reader::<File, HashMap<String, usize>>(file);
        assert!(load.is_ok());
        from_ser(load.unwrap());
    }
}
