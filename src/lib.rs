extern crate rustc_hash;

use crate::fill::Fill;
use fill::simple::Filler;
use rustc_hash::FxHashMap;
use trie::Trie;

use crate::crossword::Crossword;

use crate::crossword::Direction;
use std::fs::File;

pub mod crossword;
pub mod fill;
pub mod parse;
pub mod trie;

pub fn fill_crossword_with_default_wordlist(crossword: &Crossword) -> Result<Crossword, String> {
    let trie = Trie::load_default().expect("Failed to load trie");
    Filler::new(&trie).fill(crossword)
}
