use crate::crossword::Crossword;
use crate::crossword::CrosswordWordIterator;
use crate::crossword::Direction;
use crate::trie::Trie;
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

pub fn default_words() -> Vec<String> {
    let file = File::open("wordlist.json").unwrap();
    serde_json::from_reader(file).expect("JSON was not well-formatted")
}

pub fn index_words(raw_data: Vec<String>) -> (HashMap<(char, char), usize>, Trie) {
    let bigram = bigrams(&raw_data);
    let trie = Trie::build(raw_data);
    (bigram, trie)
}
